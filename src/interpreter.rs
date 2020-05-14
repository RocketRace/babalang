use crate::token::{Noun, Conditional, Prefix, Property};
use crate::instruction::{Instruction, Simple};
use crate::statement::Target;
use crate::error_handler::{ErrorType, throw_error, throw_error_str};
use crate::object::{
    Object, Type, Level, Image, You, Group, Empty, Reference, ImageInstance,
    EMPTY, LEVEL
};

use std::collections::HashMap;
use std::io::{stdin, stdout, Read, Write};
use std::process::exit;

/// Executes a Babalang AST in the global scope.
pub fn exec<'a>(ast: &'a [Instruction], identifiers: &HashMap<usize, String>) {
    let mut locals: HashMap<usize, Object> = HashMap::new();
    let mut globals: HashMap<usize, Object> = HashMap::new();
    globals.insert(0, EMPTY);
    globals.insert(1, LEVEL);
    // Scopes 0, 1 and 2 are reserved
    // 0 is used to refer to the program scope
    // 1 signifies that a function scope has been exited
    // 2 signifies that a scope should not be exited
    exec_with(ast, &mut locals, &mut globals, PRG_SCOPE, identifiers);
}

pub const PRG_SCOPE: usize = 0;
pub const NO_BREAK: usize = 1;
pub const _UNUSED_SCOPE: usize = 2;

/// Executes a Babalang AST with a limited scope.
/// 
/// `locals` is a slice of identifiers if this is in a function scope.
/// 
/// `globals` is the set of floating variables. 
/// 
/// The built-in EMPTY, LEVEL and IMAGE objects will always be accessible in all scopes.
fn exec_with<'a>(
    ast: &'a [Instruction], 
    locals: &mut HashMap<usize, Object>,
    globals: &mut HashMap<usize, Object>,
    _scope: usize, // Possible useful for error messages
    identifiers: &HashMap<usize, String>
) -> (usize, Option<Object>) {
    let (mut return_scope, mut return_value) = (NO_BREAK, None);
    for instruction in ast {
        match instruction {
            Instruction::Level(level) => {
                let mut new_callback = level.instructions.to_owned();
                new_callback.push(Instruction::Simple(Simple::MakeValue(level.identifier, 0)));
                let obj = Object {
                    reference_count: 0,
                    obj_type: Type::Level(Level {
                        identifier: level.identifier,
                        arguments: level.arguments.to_owned(),
                        parameters: Vec::new(),
                        callback: new_callback
                    })
                };
                initialize(level.identifier, obj, level.float, locals, globals, identifiers);
            },
            Instruction::Image(image) => {
                let attributes: HashMap<usize, Option<Object>> = image.attributes.iter()
                    .map(|&attr| (attr, None))
                    .collect();
                let mut new_callback = image.constructor.instructions.to_vec();
                new_callback.push(
                    Instruction::Simple(
                        Simple::MakeValue(image.identifier, image.constructor.arguments[0])
                    )
                );
                let obj = Object { 
                    reference_count: 0,
                    obj_type: Type::Image(Image {
                        identifier: image.identifier,
                        attribute_pointer: 0,
                        attributes: attributes,
                        constructor: Level {
                            identifier: image.identifier,
                            arguments: image.constructor.arguments.to_owned(),
                            parameters: Vec::new(),
                            callback: new_callback
                        }
                    })
                };
                initialize(image.identifier, obj, image.float, locals, globals, identifiers);
            },
            Instruction::Tele(tele) => {
                loop {
                    let (result, returns) = exec_with(
                        &tele.instructions, 
                        locals, 
                        globals, 
                        tele.identifier, 
                        identifiers
                    );
                    return_value = returns;
                    if result == NO_BREAK {
                        continue;    
                    }
                    else if result == tele.identifier {
                        break;
                    }
                    else {
                        return_scope = result;
                        return (return_scope, return_value)
                    }
                }
            },
            Instruction::Complex(complex) => {
                let conditional_id = match complex.instruction {
                    Simple::Text(id) => Some(id),
                    Simple::Word(id) => Some(id),
                    Simple::Win(id) => Some(id),
                    Simple::Defeat(id) => Some(id),
                    Simple::IsValue(id, _, _) => Some(id),
                    Simple::IsSum(id, _, _) => Some(id),
                    Simple::MimicReference(id, _) => Some(id),
                    Simple::IsEmpty(id) => Some(id),
                    Simple::Move(id, _) => Some(id),
                    Simple::Turn(id, _) => Some(id),
                    Simple::Fall(id, _) => Some(id),
                    Simple::More(id, _) => Some(id),
                    Simple::Right(id, _) => Some(id),
                    Simple::Up(id, _) => Some(id),
                    Simple::Left(id, _) => Some(id),
                    Simple::Down(id, _) => Some(id),
                    Simple::Shift(id, _) => Some(id),
                    Simple::Sink(id) => Some(id),
                    Simple::Swap(id) => Some(id),
                    Simple::HasValue(id, _) => Some(id),
                    Simple::MakeValue(id, _) => Some(id),
                    Simple::Power(id) => Some(id),
                    Simple::FearTele(id, _) => Some(id),
                    Simple::FollowAttribute(id, _) => Some(id),
                    Simple::EatValue(id, _) => Some(id),
                    _ => None
                };
                if let Some(source_id) = conditional_id {
                    if let Some(source) = find_ref(&source_id, locals, globals, identifiers) {
                        let mut complete = true;
                        if let Some(conds) = &complex.conditions {
                            match conds.cond_type {
                                Conditional::On => {
                                    for target in conds.targets.iter() {
                                        if let Target::Noun(Noun::Identifier(target_id)) = target {
                                            if let Some(obj) = find_ref(target_id, locals, globals, identifiers) {
                                                if !((obj.obj_type == source.obj_type) ^ conds.sign) {
                                                    complete = false;
                                                }
                                            }
                                        }
                                        else if let Target::Noun(Noun::All) = target {
                                            if let Type::You(you) = source.obj_type {
                                                for (_, loc_obj) in locals.iter() {
                                                    if let Type::You(target_you) = loc_obj.obj_type {
                                                        if !((you.x == target_you.x && you.y == target_you.y) ^ conds.sign) {
                                                            complete = false;
                                                        }
                                                    }
                                                }
                                                for (_, loc_obj) in globals.iter() {
                                                    if let Type::You(target_you) = loc_obj.obj_type {
                                                        if !((you.x == target_you.x && you.y == target_you.y) ^ conds.sign) {
                                                            complete = false;
                                                        }
                                                    }
                                                }
                                            }
                                            else {
                                                throw_error_str(ErrorType::TypeError, "Invalid target for ON conditional");
                                                complete = false;
                                            }
                                        }
                                        else {
                                            throw_error_str(ErrorType::TypeError, "Invalid target for ON conditional");
                                            complete = false;
                                        }
                                    }
                                },
                                Conditional::Near => {
                                    for target in conds.targets.iter() {
                                        if let Target::Noun(Noun::Identifier(target_id)) = target {
                                            if let Some(obj) = find_ref(target_id, locals, globals, identifiers) {
                                                if is_same_type(obj, source) {
                                                    if conds.sign {
                                                        complete = false;
                                                    }
                                                }
                                                else {
                                                    if !conds.sign {
                                                        complete = false;
                                                    }
                                                }
                                            }
                                        }
                                        else if let Target::Noun(Noun::All) = target {
                                            for (_, obj) in locals.iter() {
                                                if is_same_type(obj, source) {
                                                    if conds.sign {
                                                        complete = false;
                                                    }
                                                }
                                                else {
                                                    if !conds.sign {
                                                        complete = false;
                                                    }
                                                }
                                            }
                                            for (_, obj) in globals.iter() {
                                                if is_same_type(obj, source) {
                                                    if conds.sign {
                                                        complete = false;
                                                    }
                                                }
                                                else {
                                                    if !conds.sign {
                                                        complete = false;
                                                    }
                                                }
                                            }
                                        }
                                        else if let Target::Noun(Noun::Empty) = target {
                                            if let Type::Empty(_) = source.obj_type {
                                                if conds.sign {
                                                    complete = false;
                                                }
                                            }
                                            else {
                                                if !conds.sign {
                                                    complete = false;
                                                }
                                            }
                                        }
                                        else if let Target::Noun(Noun::Level) = target {
                                            if let Type::Level(_) = source.obj_type {
                                                if conds.sign {
                                                    complete = false;
                                                }
                                            }
                                            else {
                                                if !conds.sign {
                                                    complete = false;
                                                }
                                            }
                                        }
                                        else if let Target::Noun(Noun::Image) = target {
                                            if let Type::Image(_) = source.obj_type {
                                                if conds.sign {
                                                    complete = false;
                                                }
                                            }
                                            else if let Type::ImageInstance(_) = source.obj_type {
                                                if conds.sign {
                                                    complete = false;
                                                }
                                            }
                                            else {
                                                if !conds.sign {
                                                    complete = false;
                                                }
                                            }
                                        }
                                        else {
                                            complete = false;
                                            throw_error_str(ErrorType::TypeError, "Invalid target for NEAR conditional");
                                        }
                                    }
                                },
                                Conditional::Facing => {
                                    for target in conds.targets.iter() {
                                        if let Target::Noun(Noun::Identifier(target_id)) = target {
                                            if let Some(obj) = find_ref(target_id, locals, globals, identifiers) {
                                                if let Type::You(you) = source.obj_type {
                                                    if let Type::You(target_obj) = obj.obj_type {
                                                        if !((you < target_obj) ^ conds.sign) {
                                                            complete = false;
                                                        }
                                                    }
                                                    else {
                                                        complete = false;
                                                        throw_error_str(ErrorType::TypeError, "Invalid target for FACING conditional");
                                                    }
                                                }
                                                else if let Type::Group(group) = &source.obj_type {
                                                    if let Type::Group(target_obj) = &obj.obj_type {
                                                        if !((group < target_obj) ^ conds.sign) {
                                                            complete = false;
                                                        }
                                                    }
                                                    else {
                                                        complete = false;
                                                        throw_error_str(ErrorType::TypeError, "Invalid target for FACING conditional");
                                                    }
                                                }
                                                else {
                                                    complete = false;
                                                    throw_error_str(ErrorType::TypeError, "Invalid subject for FACING conditional");
                                                }
                                            }
                                        }
                                        else if let Target::Noun(Noun::All) = target {
                                            for (_, obj) in locals.iter() {
                                                if let Type::You(you) = source.obj_type {
                                                    if let Type::You(target_obj) = obj.obj_type {
                                                        if !((you < target_obj) ^ conds.sign) {
                                                            complete = false;
                                                        }
                                                    }
                                                    else {
                                                        complete = false;
                                                        throw_error_str(ErrorType::TypeError, "Invalid target for FACING conditional");
                                                    }
                                                }
                                                else if let Type::Group(group) = &source.obj_type {
                                                    if let Type::Group(target_obj) = &obj.obj_type {
                                                        if !((group < target_obj) ^ conds.sign) {
                                                            complete = false;
                                                        }
                                                    }
                                                    else {
                                                        complete = false;
                                                        throw_error_str(ErrorType::TypeError, "Invalid target for FACING conditional");
                                                    }
                                                }
                                                else {
                                                    complete = false;
                                                    throw_error_str(ErrorType::TypeError, "Invalid subject for FACING conditional");
                                                }
                                            }
                                            for (_, obj) in globals.iter() {
                                                if let Type::You(you) = source.obj_type {
                                                    if let Type::You(target_obj) = obj.obj_type {
                                                        if !((you < target_obj) ^ conds.sign) {
                                                            complete = false;
                                                        }
                                                    }
                                                    else {
                                                        complete = false;
                                                        throw_error_str(ErrorType::TypeError, "Invalid target for FACING conditional");
                                                    }
                                                }
                                                else if let Type::Group(group) = &source.obj_type {
                                                    if let Type::Group(target_obj) = &obj.obj_type {
                                                        if !((group < target_obj) ^ conds.sign) {
                                                            complete = false;
                                                        }
                                                    }
                                                    else {
                                                        complete = false;
                                                        throw_error_str(ErrorType::TypeError, "Invalid target for FACING conditional");
                                                    }
                                                }
                                                else {
                                                    complete = false;
                                                    throw_error_str(ErrorType::TypeError, "Invalid subject for FACING conditional");
                                                }
                                            }
                                        }
                                        else if let Target::Property(Property::Right) = target {
                                            if let Type::You(you) = &source.obj_type {
                                                if !((you.dir == 0) ^ conds.sign) {
                                                    complete = false;
                                                }
                                            }
                                            else {
                                                complete = false;
                                                throw_error_str(ErrorType::TypeError, "Invalid subject for FACING conditional");
                                            }
                                        }
                                        else if let Target::Property(Property::Up) = target {
                                            if let Type::You(you) = &source.obj_type {
                                                if !((you.dir == 1) ^ conds.sign) {
                                                    complete = false;
                                                }
                                            }
                                            else {
                                                complete = false;
                                                throw_error_str(ErrorType::TypeError, "Invalid subject for FACING conditional");
                                            }
                                        }
                                        else if let Target::Property(Property::Left) = target {
                                            if let Type::You(you) = &source.obj_type {
                                                if !((you.dir == 2) ^ conds.sign) {
                                                    complete = false;
                                                }
                                            }
                                            else {
                                                complete = false;
                                                throw_error_str(ErrorType::TypeError, "Invalid subject for FACING conditional");
                                            }
                                        }
                                        else if let Target::Property(Property::Down) = target {
                                            if let Type::You(you) = &source.obj_type {
                                                if !((you.dir == 3) ^ conds.sign) {
                                                    complete = false;
                                                }
                                            }
                                            else {
                                                complete = false;
                                                throw_error_str(ErrorType::TypeError, "Invalid subject for FACING conditional");
                                            }
                                        }
                                        else {
                                            complete = false;
                                            throw_error_str(ErrorType::TypeError, "Invalid target for FACING conditional");
                                        };
                                    }
                                },
                                Conditional::Without => {
                                    if let Type::Group(group) = &source.obj_type {
                                        for target in conds.targets.iter() {
                                            if let Target::Noun(Noun::Identifier(target_id)) = target {
                                                if let Some(obj) = find_ref(target_id, locals, globals, identifiers) {
                                                    let mut contains = false;
                                                    for element in group.data.iter() {
                                                        if element.obj_type == obj.obj_type {
                                                            contains = true;
                                                        }
                                                    }
                                                    if contains ^ conds.sign {
                                                        complete = false;
                                                    }
                                                }
                                            }
                                            else if let Target::Noun(Noun::All) = target {
                                                for (_, obj) in locals.iter() {
                                                    let mut contains = false;
                                                    for element in group.data.iter() {
                                                        if element.obj_type == obj.obj_type {
                                                            contains = true;
                                                        }
                                                    }
                                                    if contains ^ conds.sign {
                                                        complete = false;
                                                    }
                                                }
                                                for (_, obj) in globals.iter() {
                                                    let mut contains = false;
                                                    for element in group.data.iter() {
                                                        if element.obj_type == obj.obj_type {
                                                            contains = true;
                                                        }
                                                    }
                                                    if contains ^ conds.sign {
                                                        complete = false;
                                                    }
                                                }
                                            }
                                            else {
                                                complete = false;
                                                throw_error_str(ErrorType::TypeError, "Invalid target for WITHOUT conditional");
                                            }
                                        }
                                    }
                                    else {
                                        complete = false;
                                        throw_error_str(ErrorType::TypeError, "Invalid subject for conditional");
                                    }
                                },
                            }
                        }
                        if let Some(pref) = complex.prefix {
                            match pref.prefix {
                                Prefix::Lonely => {
                                    if let Type::You(you) = source.obj_type {
                                        if !((you.x == 0 && you.y == 0) ^ pref.sign) {
                                            complete = false;
                                        }
                                    }
                                    else if let Type::Group(group) = &source.obj_type {
                                        if !((group.data.len() == 0) ^ pref.sign) {
                                            complete = false;
                                        }
                                    }
                                    else if let Type::Empty(_) = &source.obj_type {
                                        if pref.sign {
                                            complete = false;
                                        }
                                    }
                                    else if let Type::Level(_) = &source.obj_type {
                                        if !pref.sign {
                                            complete = false;
                                        }
                                    }
                                    else if let Type::Image(img) = &source.obj_type {
                                        let mut empty = true;
                                        for (_, attr) in img.attributes.iter() {
                                            if let Some(_) = attr {
                                                empty = false;
                                            }
                                        }
                                        if !(empty ^ pref.sign) {
                                            complete = false;
                                        }
                                    }
                                    else if let Type::ImageInstance(img) = &source.obj_type {
                                        let mut empty = true;
                                        for (_, attr) in img.attributes.iter() {
                                            if let Some(_) = attr {
                                                empty = false;
                                            }
                                        }
                                        if !(empty ^ pref.sign) {
                                            complete = false;
                                        }
                                    }
                                },
                                Prefix::Idle => {
                                    if let Type::Level(level) = &source.obj_type {
                                        if !((level.arguments.len() == level.parameters.len()) ^ pref.sign) {
                                            complete = false;
                                        }
                                    }
                                    if let Type::Image(img) = &source.obj_type {
                                        if !((img.constructor.arguments.len() - 1 == img.constructor.parameters.len()) ^ pref.sign) {
                                            complete = false;
                                        }
                                    }
                                    else {
                                        complete = !pref.sign;
                                    }
                                },
                            }
                        }
                        if complete {
                            let (result, returns) = exec_simple(&complex.instruction, locals, globals, identifiers);
                            if result != NO_BREAK {
                                return_scope = result;
                                if let Some(_) = returns {
                                    return_value = returns;
                                }
                                return (return_scope, return_value);
                            }
                            if let Some(_) = returns {
                                return_value = returns;
                                return (return_scope, return_value);
                            }
                        }
                    }
                }
                else {
                    throw_error_str(
                        ErrorType::ConditionError,
                        "Conditional statements must have a single subject (not ALL, LEVEL or IMAGE)"
                    )
                }
            },
            Instruction::Simple(simple) => {
                let (result, returns) = exec_simple(simple, locals, globals, identifiers);
                if result != NO_BREAK {
                    return_scope = result;
                    if let Some(_) = returns {
                        return_value = returns;
                    }
                    return (return_scope, return_value);
                }
                if let Some(_) = returns {
                    return_value = returns;
                    return (return_scope, return_value);
                }
            },
            Instruction::NoOp => (),
            _ => ()
        }
    }
    (return_scope, return_value)
}

/// Adds an object to either the locals or the globals.
fn initialize<'a>(
    id: usize, 
    obj: Object,
    float: bool,
    locals: &mut HashMap<usize, Object>, 
    globals: &mut HashMap<usize, Object>,
    _identifiers: &HashMap<usize, String>
) {
    let extra_float = if float {
        if locals.contains_key(&id) {
            locals.remove(&id);
        }
        true
    }
    else {
        globals.contains_key(&id)
    };
    if extra_float {
        globals.insert(id, obj);
    }
    else {
        locals.insert(id, obj);
    }
}

/// Executes a single simple instruction in the provided scope.
fn exec_simple<'a>(
    simple: &Simple, 
    locals: &mut HashMap<usize, Object>, 
    globals: &mut HashMap<usize, Object>, 
    identifiers: &HashMap<usize, String>
) -> (usize, Option<Object>) {
    let (mut return_scope, mut return_value) = (NO_BREAK, None);
    match simple {
        Simple::InitYou(id, float) => {
            initialize(*id, Object { 
                reference_count: 0,
                obj_type: Type::You(You {
                    x: 0,
                    y: 0,
                    dir: 0
                })
            }, *float, locals, globals, identifiers);
        },
        Simple::InitGroup(id, float) => {
            initialize(*id, Object { 
                reference_count: 0,
                obj_type: Type::Group(Group {
                    index: 0,
                    data: Vec::new()
                })
            }, *float, locals, globals, identifiers);
        },
        Simple::Text(id) => {
            if let Some(obj) = find_ref(id, locals, globals, identifiers) {
                print_object(&obj, Some(*id));
            }
        },
        Simple::Word(id) => {
            if let Some(obj) = find_mut_ref(id, locals, globals, identifiers) {
                match &mut obj.obj_type {
                    Type::You(you) => {
                        let mut buffer: [u8; 1] = [0];
                        stdin().read(&mut buffer).unwrap();
                        if you.dir & 1 == 0 {
                            you.x = buffer[0];
                        }
                        else {
                            you.y = buffer[0];
                        }
                    },
                    Type::Group(group) => {
                        let mut buffer = String::new();
                        stdin().read_line(&mut buffer).unwrap();
                        let mut objects = buffer
                            .bytes()
                            .collect::<Vec<u8>>()
                            .iter()
                            .map(|&x| Object {
                                reference_count: 0,
                                obj_type: Type::You(You {
                                    x: x,
                                    y: 0,
                                    dir: 0
                                })
                            })
                            .collect();
                        let mut new = group.data.to_vec();
                        new.append(&mut objects);
                        group.data = new;
                        group.index += objects.len();
                    },
                    x => {
                        throw_error(
                            ErrorType::TypeError, 
                            format!("Object {} of type {} cannot be WORD", id, x),
                            Some((&[*id], identifiers))
                        );
                    }
                }
            }
        },
        Simple::Win(id) => {
            if let Some(obj) = find_ref(id, locals, globals, identifiers) {
                if let Type::You(_) = obj.obj_type {
                    exit(0);
                }
            }
        },
        Simple::Defeat(id) => {
            if let Some(obj) = find_ref(id, locals, globals, identifiers) {
                if let Type::You(_) = obj.obj_type {
                    exit(1);
                }
            }
        },
        Simple::IsValue(source_id, target_id, not) => {
            let mut glob = false;
            let mut maybe_source = if let Some(obj) = locals.get(&source_id) {
                Some(obj.clone())
            }
            else if let Some(obj) = globals.get(&source_id) {
                glob = true;
                Some(obj.clone())
            }
            else { None };
            let mut copy_value = None;
            if let Some(target) = find_ref(target_id, locals, globals, identifiers) {
                if let Some(source) = &mut maybe_source {
                    // `a is not b`, if a and b are YOU, implies:
                    // a.[value] = - b.[value], where [value] is the
                    // value in the active direction of each.
                    // Since values are u8, 255-value is the "negation"
                    if let Type::You(you) = &mut source.obj_type {
                        if let Type::You(also_you) = target.obj_type {
                            if *not {
                                you.x = 255 - also_you.x;
                                you.y = 255 - also_you.x;
                            }
                            else {
                                you.x = also_you.x;
                                you.y = also_you.y;
                            }
                            copy_value = Some(Object {
                                reference_count: 0,
                                obj_type: Type::You(You {
                                    x: you.x,
                                    y: you.y,
                                    dir: you.dir
                                })
                            })
                        }
                        else {
                            throw_error(
                                ErrorType::ObjectAlreadyDefinedError, 
                                format!("Object {} of type {} cannot be set to {}", source_id, source.obj_type, target.obj_type),
                                Some((&[*source_id], identifiers))
                            );
                        }
                    }
                    else if let Type::Empty(_) = &mut source.obj_type {
                        copy_value = Some(target.clone());
                    }
                    else {
                        throw_error(
                            ErrorType::ObjectAlreadyDefinedError, 
                            format!("Object {} of type {} cannot be set to {}", source_id, source.obj_type, target.obj_type),
                            Some((&[*source_id], identifiers))
                        );
                    }
                    if let Some(new) = copy_value {
                        if glob {
                            globals.insert(*source_id, new);
                        }
                        else {
                            locals.insert(*source_id, new);
                        }
                    }
                }
                else {
                    if let Some(target) = find_value(target_id, locals, globals, identifiers) {
                        // We clone here because the specification of IS calls for it
                        initialize(*source_id, target, false, locals, globals, identifiers);
                    }
                }
            }
        },
        Simple::MimicReference(source_id, target_id) => {
            if let Some(obj) = find_mut_ref(target_id, locals, globals, identifiers) {
                obj.reference_count += 1;
            }
            initialize(*source_id, Object {
                reference_count: 0, 
                obj_type: Type::Reference(Reference {
                    pointer: *target_id
                })
            }, false, locals, globals, identifiers);
        },
        Simple::IsEmpty(id) => {
            if let Some(obj) = locals.get_mut(id) {
                *obj = Object {
                    reference_count: 0,
                    obj_type: Type::Empty(Empty {})
                };
            }
            else if let Some(obj) = globals.get_mut(id) {
                *obj = Object {
                    reference_count: 0,
                    obj_type: Type::Empty(Empty {})
                };
            }
            else {
                locals.insert(*id, Object {
                    reference_count: 0, 
                    obj_type: Type::Empty(Empty {})
                });
            }
        },
        // Targets are guaranteed to be Noun::Identifier or Noun::All
        Simple::IsSum(source_id, targets, nots) => {
            let (mut sum_x, mut sum_y): (u8, u8) = (0, 0);
            for (target, not) in targets.iter().zip(nots.iter()) {
                if let Noun::Identifier(id) = target {
                    if let Some(target_obj) = find_value(id, locals, globals, identifiers) {
                        if let Type::You(you) = target_obj.obj_type {
                            if *not {
                                sum_x = sum_x.wrapping_sub(you.x);
                                sum_y = sum_y.wrapping_sub(you.y);
                            }
                            else {
                                sum_x = sum_x.wrapping_add(you.x);
                                sum_y = sum_y.wrapping_add(you.y);
                            }
                        }
                        else {
                            if let Noun::Identifier(id) = target {
                                throw_error(
                                    ErrorType::ObjectAlreadyDefinedError, 
                                    format!("Object {} of type {} does not support addition", id, target_obj.obj_type),
                                    Some((&[*source_id], identifiers))
                                );
                            }
                            else {
                                throw_error(
                                    ErrorType::ObjectAlreadyDefinedError, 
                                    format!("Object {:?} of type {} does not support addition", id, target_obj.obj_type),
                                    Some((&[*source_id], identifiers))
                                );
                            }
                        }
                    }
                }
                // While the use of ALL is not necessarily efficient or fast,
                // the concept itself is niche and doesn't warrant extended use
                else if let Noun::All = target {
                    let (mut all_x, mut all_y): (u8, u8) = (0, 0);
                    // Get all YOU objects in the current scope
                    let all_loc = locals.values()
                        .filter(|x| matches!(
                            x, Object { reference_count: _, obj_type: Type::You(_)}
                        ))
                        .map(|x| x.obj_type.clone());
                    let all_glob = globals.values()
                        .filter(|x| matches!(
                            x, Object { reference_count: _, obj_type: Type::You(_)}
                        ))
                        .map(|x| x.obj_type.clone());
                    // Take their sum
                    for value in all_loc {
                        if let Type::You(you) = value {
                            all_x = all_x.wrapping_add(you.x);
                            all_y = all_y.wrapping_add(you.y);
                        }
                    }
                    for value in all_glob {
                        if let Type::You(you) = value {
                            all_x = all_x.wrapping_add(you.x);
                            all_y = all_y.wrapping_add(you.y);
                        }
                    }
                    // Add the final ALL sums to our final final sum
                    if *not {
                        sum_x = sum_x.wrapping_add(all_x);
                        sum_y = sum_y.wrapping_add(all_y);
                    }
                    else {
                        sum_x = sum_x.wrapping_sub(all_x);
                        sum_y = sum_y.wrapping_sub(all_y);
                    }
                }
                else {
                    // This should never happen, but:
                    throw_error(
                        ErrorType::RuntimeError,
                        format!("Unexpected target {:?} in IsSum expression", target),
                        None
                    )
                }
            }
            // Take the result and apply that to our source object
            if let Some(obj) = locals.get_mut(&source_id) {
                if let Type::You(you_source) = &mut obj.obj_type {
                    you_source.x = sum_x;
                    you_source.y = sum_y;
                }
            }
            else if let Some(obj) = globals.get_mut(&source_id) {
                if let Type::You(you_source) = &mut obj.obj_type {
                    you_source.x = sum_x;
                    you_source.y = sum_y;
                }
            }            
            else {
                initialize(*source_id, Object {
                    reference_count: 0, 
                    obj_type: Type::You(You {
                        x: sum_x,
                        y: sum_y,
                        dir: 0
                    })
                }, false, locals, globals, identifiers);
            }
        },
        Simple::Move(id, not) => {
            if let Some(obj) = find_mut_ref(id, locals, globals, identifiers) {
                if let Type::You(you) = &mut obj.obj_type {
                    // 0 => Right
                    // 1 => Up
                    // 2 => Left
                    // 3 => Down
                    match you.dir {
                        0 => {
                            if *not {
                                if you.x == 0 {
                                    you.x = 255;
                                }
                                else {
                                    you.x -= 1;
                                }
                            }
                            else {
                                if you.x == 255 {
                                    you.x = 0;
                                }
                                else {
                                    you.x += 1;
                                }
                            }
                        },
                        1 => {
                            if *not {
                                if you.y == 0 {
                                    you.y = 255;
                                }
                                else {
                                    you.y -= 1;
                                }
                            }
                            else {
                                if you.y == 255 {
                                    you.y = 0;
                                }
                                else {
                                    you.y += 1;
                                }
                            }
                        },
                        2 => {
                            if *not {
                                if you.x == 255 {
                                    you.x = 0;
                                }
                                else {
                                    you.x += 1;
                                }
                            }
                            else {
                                if you.x == 0 {
                                    you.x = 255;
                                }
                                else {
                                    you.x -= 1;
                                }
                            }
                        },
                        3 => {
                            if *not {
                                if you.y == 255 {
                                    you.y = 0;
                                }
                                else {
                                    you.y += 1;
                                }
                            }
                            else {
                                if you.y == 0 {
                                    you.y = 255;
                                }
                                else {
                                    you.y -= 1;
                                }
                            }
                        },
                        // This should never happen
                        _ => ()
                    }
                }
                else {
                    throw_error(
                        ErrorType::TypeError, 
                        format!("Object {} of type {} cannot be MOVE", id, obj.obj_type),
                        Some((&[*id], identifiers))
                    );
                }
            }
        },
        Simple::Turn(id, not) => {
            if let Some(obj) = find_mut_ref(id, locals, globals, identifiers) {
                if let Type::You(you) = &mut obj.obj_type {
                    if *not {
                        if you.dir == 0 {
                            you.dir = 3;
                        }
                        else {
                            you.dir -= 1;
                        }
                    }
                    else {
                        if you.dir == 3 {
                            you.dir = 0;
                        }
                        else {
                            you.dir += 1;
                        }
                    }
                }
                else {
                    throw_error(
                        ErrorType::TypeError, 
                        format!("Object {} of type {} cannot be TURN", id, obj.obj_type),
                        Some((&[*id], identifiers))
                    );
                }
            }
        },
        Simple::Fall(id, not) => {
            if let Some(obj) = find_mut_ref(id, locals, globals, identifiers) {
                if let Type::You(you) = &mut obj.obj_type {
                    if *not {
                        if you.dir & 1 == 0 {
                            you.x = 0;
                        }
                        else {
                            you.y = 0;
                        }
                    }
                    else {
                        if you.dir & 1 == 0 {
                            you.x = 255;
                        }
                        else {
                            you.y = 255;
                        }
                    }
                }
                else {
                    throw_error(
                        ErrorType::TypeError, 
                        format!("Object {} of type {} cannot be FALL", id, obj.obj_type),
                        Some((&[*id], identifiers))
                    );
                }
            }
        },
        Simple::More(id, not) => {
            if let Some(obj) = find_mut_ref(id, locals, globals, identifiers) {
                if let Type::You(you) = &mut obj.obj_type {
                    if *not {
                        if you.dir & 1 == 0 {
                            you.x = you.x >> 1;
                        }
                        else {
                            you.y = you.y >> 1;
                        }
                    }
                    else {
                        if you.dir & 1 == 0 {
                            you.x = you.x << 1;
                        }
                        else {
                            you.y = you.y << 1;
                        }
                    }
                }
                else {
                    throw_error(
                        ErrorType::TypeError, 
                        format!("Object {} of type {} cannot be MORE", id, obj.obj_type),
                        Some((&[*id], identifiers))
                    );
                }
            }
        },
        Simple::Right(id, not) => {
            if let Some(obj) = find_mut_ref(id, locals, globals, identifiers) {
                if let Type::You(you) = &mut obj.obj_type {
                    if *not {
                        you.dir = 2;
                    }
                    else {
                        you.dir = 0;
                    }
                }
                else {
                    throw_error(
                        ErrorType::TypeError, 
                        format!("Object {} of type {} cannot be RIGHT", id, obj.obj_type),
                        Some((&[*id], identifiers))
                    );
                }
            }
        },
        Simple::Up(id, not) => {
            if let Some(obj) = find_mut_ref(id, locals, globals, identifiers) {
                if let Type::You(you) = &mut obj.obj_type {
                    if *not {
                        you.dir = 3;
                    }
                    else {
                        you.dir = 1;
                    }
                }
                else {
                    throw_error(
                        ErrorType::TypeError, 
                        format!("Object {} of type {} cannot be UP", id, obj.obj_type),
                        Some((&[*id], identifiers))
                    );
                }
            }
        },
        Simple::Left(id, not) => {
            if let Some(obj) = find_mut_ref(id, locals, globals, identifiers) {
                if let Type::You(you) = &mut obj.obj_type {
                    if *not {
                        you.dir = 0;
                    }
                    else {
                        you.dir = 2;
                    }
                }
                else {
                    throw_error(
                        ErrorType::TypeError, 
                        format!("Object {} of type {} cannot be LEFT", id, obj.obj_type),
                        Some((&[*id], identifiers))
                    );
                }
            }
        },
        Simple::Down(id, not) => {
            if let Some(obj) = find_mut_ref(id, locals, globals, identifiers) {
                if let Type::You(you) = &mut obj.obj_type {
                    if *not {
                        you.dir = 1;
                    }
                    else {
                        you.dir = 3;
                    }
                }
                else {
                    throw_error(
                        ErrorType::TypeError, 
                        format!("Object {} of type {} cannot be DOWN", id, obj.obj_type),
                        Some((&[*id], identifiers))
                    );
                }
            }
        },
        Simple::AllMove(not) => {
            exec_all(&Simple::Move, *not, locals, globals, identifiers);
        },
        Simple::AllTurn(not) => {
            exec_all(&Simple::Turn, *not, locals, globals, identifiers);
        },
        Simple::AllFall(not) => {
            exec_all(&Simple::Fall, *not, locals, globals, identifiers);
        },
        Simple::AllMore(not) => {
            exec_all(&Simple::More, *not, locals, globals, identifiers);
        },
        Simple::AllRight(not) => {
            exec_all(&Simple::Right, *not, locals, globals, identifiers);
        },
        Simple::AllUp(not) => {
            exec_all(&Simple::Up, *not, locals, globals, identifiers);
        },
        Simple::AllLeft(not) => {
            exec_all(&Simple::Left, *not, locals, globals, identifiers);
        },
        Simple::AllDown(not) => {
            exec_all(&Simple::Down, *not, locals, globals, identifiers);
        },
        Simple::Shift(id, not) => {
            if let Some(obj) = find_mut_ref(id, locals, globals, identifiers) {
                if let Type::Group(group) = &mut obj.obj_type {
                    if *not {
                        if group.index == 0 {
                            group.index = group.data.len() - 1;
                        }
                        else {
                            group.index -= 1;
                        }
                    }
                    else {
                        if group.index == group.data.len() - 1 {
                            group.index = 0;
                        }
                        else {
                            group.index += 1;
                        }

                    }
                }
                else {
                    throw_error(
                        ErrorType::TypeError, 
                        format!("Object {} of type {} cannot be SHIFT", id, obj.obj_type),
                        Some((&[*id], identifiers))
                    );
                }
            }
        },
        Simple::Sink(id) => {
            if let Some(obj) = find_mut_ref(id, locals, globals, identifiers) {
                if let Type::Group(group) = &mut obj.obj_type {
                    group.data.pop();
                }
                else {
                    throw_error(
                        ErrorType::TypeError, 
                        format!("Object {} of type {} cannot be SINK", id, obj.obj_type),
                        Some((&[*id], identifiers))
                    );
                }
            }
        },
        Simple::Swap(id) => {
            if let Some(obj) = find_mut_ref(id, locals, globals, identifiers) {
                if let Type::Group(group) = &mut obj.obj_type {
                    let last = group.data.len() - 1;
                    group.data.swap(group.index, last);
                }
                else {
                    throw_error(
                        ErrorType::TypeError, 
                        format!("Object {} of type {} cannot be SWAP", id, obj.obj_type),
                        Some((&[*id], identifiers))
                    );
                }
            }
        },
        Simple::HasValue(source_id, target_id) => {
            let maybe_target = find_value(target_id, locals, globals, identifiers);
            if let Some(obj) = find_mut_ref(source_id, locals, globals, identifiers) {
                if let Type::Group(group) = &mut obj.obj_type {
                    if let Some(target) = maybe_target {
                        group.data.push(target);
                    }
                }
                else if let Type::Level(level) = &mut obj.obj_type {
                    if let Some(target) = maybe_target {
                        level.parameters.push(target);
                    }
                }
                else if let Type::Image(image) = &mut obj.obj_type {
                    if let Some(target) = maybe_target {
                        image.constructor.parameters.push(target);
                    }
                }
            }
        },
        Simple::MakeValue(source_id, target_id) => {
            let collection_type = if let Some(obj) = find_ref(source_id, locals, globals, identifiers) {
                if let Type::Group(_) = &obj.obj_type {
                    1
                }
                else if let Type::Level(_) = &obj.obj_type {
                    2
                }
                else if let Type::Image(_) = &obj.obj_type {
                    3
                }
                else if let Type::ImageInstance(_) = &obj.obj_type {
                    3
                }
                else {
                    0
                }
            }
            else {0};
            match collection_type {
                1 => {
                    let maybe_element = if let Some(obj) = find_mut_ref(source_id, locals, globals, identifiers) {
                        if let Type::Group(group) = &mut obj.obj_type {
                            group.data.pop()
                        } else {None}
                    } else {None};
                    if let Some(obj) = maybe_element {
                        locals.insert(*target_id, obj);
                    }
                },
                2 => {
                    if let Some(obj) = find_value(target_id, locals, globals, identifiers) {
                        return_value = Some(obj);
                        return_scope = *source_id;
                    }
                }
                3 => {
                    let maybe_attr = if let Some(obj) = find_mut_ref(source_id, locals, globals, identifiers) {
                        if let Type::Image(image) = &mut obj.obj_type {
                            image.attributes[&image.attribute_pointer].clone()
                        } 
                        else if let Type::ImageInstance(image) = &mut obj.obj_type {
                            image.attributes[&image.attribute_pointer].clone()
                        } else {None}
                    } else {None};
                    if let Some(obj) = maybe_attr {
                        locals.insert(*target_id, obj);
                    }
                },
                _ => {
                    if let Some(obj) = find_ref(source_id, locals, globals, identifiers) {
                        throw_error(
                            ErrorType::TypeError, 
                            format!("Object {} of type {} cannot MAKE anything", source_id, obj.obj_type),
                            Some((&[*source_id], identifiers))
                        );
                    }
                }
            }

        },
        Simple::Power(id) => {
            // This line is here to avoid borrow conflicts
            let mut new_globals = globals.clone();
            let mut new_locals = locals.clone();
            let mut ret_val = None;
            let self_ref = find_value(id, locals, globals, identifiers);
            let glob = if let Some(_) = globals.get(id) {true} else {false};
            if let Some(obj) = find_mut_ref(id, locals, globals, identifiers) {
                if let Type::Level(level) = &mut obj.obj_type {
                    if level.arguments.len() == level.parameters.len() {
                        for (arg, param) in level.arguments.iter().zip(level.parameters.iter()) {
                            new_locals.insert(*arg, param.clone());
                        }
                        new_locals.insert(level.identifier, self_ref.unwrap());
                        let (_, fn_ret_val) = exec_with(
                            &level.callback, 
                            &mut new_locals, 
                            &mut new_globals,
                            *id, 
                            identifiers
                        );
                        ret_val = fn_ret_val
                    }
                    else {
                        throw_error(
                            ErrorType::ArgumentError, 
                            format!(
                                "Expected {} arguments when calling POWER on object {} of type LEVEL, got {} arguments",
                                level.arguments.len(), 
                                id, 
                                level.parameters.len()
                            ),
                            Some((&[*id], identifiers))
                        );
                    }
                }
                else if let Type::Image(image) = &mut obj.obj_type {
                    if image.constructor.arguments.len() - 1 == image.constructor.parameters.len() {
                        for (arg, param) in image.constructor.arguments
                            .iter()
                            .skip(1)
                            .zip(image.constructor.parameters.iter()) 
                        {
                            new_locals.insert(*arg, param.clone());
                        }
                        new_locals.insert(image.identifier, Object {
                            reference_count: 0, obj_type: Type::Level(image.constructor.clone()
                        )});
                        new_locals.insert(
                            image.constructor.arguments[0], 
                            Object {
                                reference_count: 0, obj_type: Type::ImageInstance(ImageInstance {
                                    class: image.identifier,
                                    attribute_pointer: image.attribute_pointer,
                                    attributes: image.attributes.clone(),
                                })
                            }
                        );
                        let (_, fn_ret_val) = exec_with(
                            &image.constructor.callback, 
                            &mut new_locals, 
                            &mut new_globals,
                            *id, 
                            identifiers
                        );
                        ret_val = fn_ret_val
                    }
                    else {
                        throw_error(
                            ErrorType::ArgumentError, 
                            format!(
                                "Expected {} arguments when calling POWER on object {} of type LEVEL, got self + {} arguments",
                                image.constructor.arguments.len(), 
                                id, 
                                image.constructor.parameters.len()
                            ),
                            Some((&[*id], identifiers))
                        );
                    }
                }
                else {
                    throw_error(
                        ErrorType::TypeError, 
                        format!("Object {} of type {} cannot be POWER", id, obj.obj_type),
                        Some((&[*id], identifiers))
                    );
                }
            }
            if let Some(obj) = ret_val {
                if glob {
                    globals.insert(*id, obj);
                }   
                else {
                    locals.insert(*id, obj);
                }
            }
        },
        Simple::FearTele(source_id, target_id) => {
            // Set the return scope (the final scope to be broken from)
            // to the specified tele loop (if it exists)
            if let Some(_valid_obj) = find_ref(source_id, locals, globals, identifiers) {
                return_scope = *target_id;
            }
        },
        Simple::FollowAttribute(source_id, attr_id) => {
            if let Some(obj) = find_mut_ref(source_id, locals, globals, identifiers) {
                if let Type::Image(image) = &mut obj.obj_type {
                    image.attribute_pointer = *attr_id;
                }
                else if let Type::ImageInstance(image) = &mut obj.obj_type {
                    image.attribute_pointer = *attr_id;
                }
                else {
                    throw_error(
                        ErrorType::TypeError, 
                        format!("Object {} of type {} cannot FOLLOW anything", source_id, obj.obj_type),
                        Some((&[*source_id], identifiers))
                    );
                }
            }
        },
        Simple::EatValue(source_id, target_id) => {
            let maybe_target = find_value(target_id, locals, globals, identifiers);
            if let Some(obj) = find_mut_ref(source_id, locals, globals, identifiers) {
                if let Type::Image(image) = &mut obj.obj_type {
                    if let Some(target) = maybe_target {
                        image.attributes.insert(image.attribute_pointer, Some(target));
                    }
                }
                else if let Type::ImageInstance(image) = &mut obj.obj_type {
                    if let Some(target) = maybe_target {
                        image.attributes.insert(image.attribute_pointer, Some(target));
                    }
                }
                else {
                    throw_error(
                        ErrorType::TypeError, 
                        format!("Object {} of type {} cannot EAT anything", source_id, obj.obj_type),
                        Some((&[*source_id], identifiers))
                    );
                }
            }
        }
    }
    (return_scope, return_value)
}

/// Searches for an object in the locals and globals provided. 
/// If found, returns a reference to the object.
/// If not found, throws an error and returns None. 
fn find_ref<'a>(
    id: &usize, 
    locals: &'a HashMap<usize, Object>, 
    globals: &'a HashMap<usize, Object>,
    identifiers: &HashMap<usize, String>
) -> Option<&'a Object> {
    if let Some(obj) = locals.get(&id) {
        if let Type::Reference(reference) = obj.obj_type {
            find_ref(&reference.pointer, locals, globals, identifiers)
        }
        else {
            Some(obj)
        }
    }
    else if let Some(obj) = globals.get(&id) {
        if let Type::Reference(reference) = obj.obj_type {
            find_ref(&reference.pointer, locals, globals, identifiers)
        }
        else {
            Some(obj)
        }
    }
    else {
        throw_error(
            ErrorType::ObjectNotDefinedError, 
            format!("Object {} is not defined in the local or global scopes", id),
            Some((&[*id], identifiers))
        );
        None
    }
}

/// Searches for an object in the locals and globals provided. 
/// If found, returns the cloned value of the object.
/// If not found, throws an error and returns None. 
/// 
/// 
fn find_value<'a>(
    id: &usize, 
    locals: &'a HashMap<usize, Object>, 
    globals: &'a HashMap<usize, Object>,
    identifiers: &HashMap<usize, String>
) -> Option<Object> {
    if let Some(obj) = locals.get(&id) {
        if let Type::Reference(reference) = obj.obj_type {
            find_value(&reference.pointer, locals, globals, identifiers)
        }
        else {
            Some(obj.clone())
        }
    }
    else if let Some(obj) = globals.get(&id) {
        if let Type::Reference(reference) = obj.obj_type {
            find_value(&reference.pointer, locals, globals, identifiers)
        }
        else {
            Some(obj.clone())
        }
    }
    else {
        throw_error(
            ErrorType::ObjectNotDefinedError, 
            format!("Object {} is not defined in the local or global scopes", id),
            Some((&[*id], identifiers))
        );
        None
    }
}

/// Searches for an object in the locals and globals provided. 
/// If found, returns a mutable reference to the object.
/// If not found, throws an error and returns None. 
fn find_mut_ref<'a>(
    id: &usize, 
    locals: &'a mut HashMap<usize, Object>, 
    globals: &'a mut HashMap<usize, Object>,
    identifiers: &HashMap<usize, String>
) -> Option<&'a mut Object> {
    // This is rearranged to avoid borrowing locals/globals as mutable twice.
    // Instead of doing that, we check for a Reference, and overwrite the 
    // current object ID to the ID being pointed to and call this function again appropriately.
    let (referenced, glob, ref_id) = if let Some(obj) = locals.get_mut(id) {
        if let Type::Reference(reference) = obj.obj_type {
            (true, false, reference.pointer)
        }
        else {
            (false, false, *id)
        }
    }
    else if let Some(obj) = globals.get_mut(id) {
        if let Type::Reference(reference) = obj.obj_type {
            (true, true, reference.pointer)
        }
        else {
            (false, true, *id)
        }
    }
    else {
        // Not found
        throw_error(
            ErrorType::ObjectNotDefinedError, 
            format!("Object {} is not defined in the local or global scopes", id),
            Some((&[*id], identifiers))
        );
        return None;
    };
    // Evaluate references
    if referenced {
        find_mut_ref(&ref_id, locals, globals, identifiers)
    }
    else {
        // Get the object normally
        if glob {
            globals.get_mut(&ref_id)
        }
        else {
            locals.get_mut(&ref_id)
        }
    }
}

/// Executes a simple YOU instruction for every YOU object in the current scope.
fn exec_all(
    simple_factory: &dyn Fn(usize, bool) -> Simple,
    not: bool,
    locals: &mut HashMap<usize, Object>,
    globals: &mut HashMap<usize, Object>,
    identifiers: &HashMap<usize, String>
) {
    // Get all YOU keys in the current scope
    let all_loc: Vec<usize> = locals.iter()
        .filter(|(_, v)| matches!(
            v, Object { reference_count: _, obj_type: Type::You(_)}
        ))
        .map(|(&k, _)| k)
        .collect();
    let all_glob: Vec<usize> = globals.iter()
        .filter(|(_, v)| matches!(
            v, Object { reference_count: _, obj_type: Type::You(_)}
        ))
        .map(|(&k, _)| k)
        .collect();
    for id in all_loc {
        exec_simple(&simple_factory(id, not), locals, globals, identifiers);
    }
    for id in all_glob {
        exec_simple(&simple_factory(id, not), locals, globals, identifiers);
    }
}

/// Prints an object's data (`X IS TEXT`) to stdout.
/// Throws an error if the object type doesn't support TEXT.
/// 
/// For YOU objects, prints the 8-bit character associated
/// with the object's active axis (active axis === right-left VS up-down?).
/// 
/// For GROUP objects, recursively calls `print_object` on each
/// element of the group.
/// 
/// For EMPTY objects, does nothing.
/// 
/// For LEVEL / IMAGE objects, throws a TypeError.
fn print_object(obj: &Object, id: Option<usize>) {
    match &obj.obj_type {
        Type::You(you) => {
            if you.dir & 1 == 0 {
                // Unwrap will catch syscall errors
                let mut out = stdout();
                out.write(&[you.x]).unwrap();
                out.flush().unwrap();
            }
            else {
                let mut out = stdout();
                out.write(&[you.y]).unwrap();
                out.flush().unwrap();
            }
        },
        Type::Group(group) => {
            for object in &group.data {
                print_object(&object, None);
            }
        },
        x => {
            if let Some(i) = id {
                throw_error(
                    ErrorType::TypeError, 
                    format!("Object {} of type {} cannot be TEXT", i, x),
                    None
                );
            }
            else {
                throw_error(
                    ErrorType::TypeError, 
                    format!("[Unnamed Object] (element of GROUP) of type {} cannot be TEXT", x),
                    None
                );
            }
        }
    }
}

/// Checks if the two objects are of the same variant.
fn is_same_type(first: &Object, other: &Object) -> bool {
    if let Type::You(_) = first.obj_type {
        if let Type::You(_) = other.obj_type {
            true
        }
        else {
            false
        }
    }
    else if let Type::Group(_) = first.obj_type {
        if let Type::Group(_) = other.obj_type {
            true
        }
        else {
            false
        }
    }
    else if let Type::Level(_) = first.obj_type {
        if let Type::Level(_) = other.obj_type {
            true
        }
        else {
            false
        }
    }
    else if let Type::Image(_) = first.obj_type {
        if let Type::Image(_) = other.obj_type {
            true
        }
        else {
            false
        }
    }
    else if let Type::ImageInstance(img) = &first.obj_type {
        if let Type::ImageInstance(other_img) = &other.obj_type {
            img.class == other_img.class
        }
        else {
            false
        }
    }
    else {
        false
    }
}
