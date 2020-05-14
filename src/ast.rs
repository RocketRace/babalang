use crate::instruction::{Instruction, Simple, Tele, Level, Image, validate, conditions};
use crate::statement::{Statement, Target};
use crate::token::{Verb, Property, Noun};
use crate::error_handler::{throw_error, ErrorType, throw_error_str};

use std::collections::HashMap;

/// Parses a stream of statements into instructions.
pub fn parse<'a>(statements: &'a [Statement], identifiers: &HashMap<usize, String>) -> Vec<Instruction> {
    let (inner, _inner_last) = parse_inner(statements, None, identifiers);
    inner
}

/// Pushes an instruction to a vector, unless it is a no-op.
fn push_nonempty<'a>(vec: &mut Vec<Instruction>, instruction: Instruction) {
    if let Instruction::NoOp = instruction {} else {
        vec.push(instruction);
    }
}

/// Parses a stream of statements into instructions.
/// 
/// If `scope` is given, will only parse until the given scope is exited
/// via `[id=scope] IS DONE`. Otherwise, will read until `ALL IS DONE` is 
/// encountered, or until the stream ends.
fn parse_inner<'a>(
    statements: &'a [Statement], 
    scope: Option<usize>,
    identifiers: &HashMap<usize, String>
) -> (Vec<Instruction>, usize) {
    let mut out = Vec::new();
    let mut iter = statements.iter().enumerate();
    let mut last = 0;
    // This `for` loop is desugared to allow for elements to be skipped
    while let Some((i, statement)) = iter.next() {
        last = i;
        let action_type = statement.action_type;
        match action_type {
            Verb::Is => {
                if let Some(target) = statement.action_target {
                    if let Target::Property(prop) = target {
                        match prop {
                            // Exit a scope.
                            // Can be either <SCOPE> IS DONE (given a scope),
                            // or ALL IS DONE (in program scope, i.e. None)
                            Property::Done => {
                                match conditions(statement) {
                                    (None, None) => {
                                        match statement.subject {
                                            Noun::Identifier(id) => {
                                                match scope {
                                                    // X IS TELE/LEVEL/IMAGE, ..., X IS DONE
                                                    Some(value) if value == id => {
                                                        return (out, last);
                                                    }
                                                    _ => {
                                                        throw_error(
                                                            ErrorType::InstructionParserError, 
                                                            format!(
                                                                "Cannot exit out of {:?} when in global scope",
                                                                statement.subject 
                                                            ),
                                                            Some((&[id], identifiers))
                                                        )
                                                    }
                                                }
                                            },
                                            Noun::All => {
                                                if let None = scope {
                                                    // ALL IS DONE
                                                    if let false = statement.action_sign {
                                                        return (out, last);
                                                    }
                                                    // ALL IS NOT DONE
                                                    else {
                                                        throw_error_str(
                                                            ErrorType::InstructionValidationError, 
                                                            "Cannot call ALL IS DONE"    
                                                        )
                                                    }
                                                }
                                                else {
                                                    throw_error_str(
                                                        ErrorType::InstructionParserError, 
                                                        "Unexpected ALL IS DONE in inner scope"
                                                    )
                                                }
                                            },
                                            _ => {
                                                throw_error(
                                                    ErrorType::InstructionValidationError, 
                                                    format!("Cannot exit out of {:?}", statement.subject),
                                                    None
                                                )
                                            }
                                        }
                                    },
                                    _ => {
                                        throw_error_str(
                                            ErrorType::InstructionValidationError, 
                                            "Cannot call IS DONE conditionally"    
                                        )
                                    }
                                }
                            },
                            // Initialize primitive objects
                            Property::You => push_nonempty(&mut out, validate("InitYou", statement, identifiers)),
                            Property::You2 => push_nonempty(&mut out, validate("InitYou2", statement, identifiers)),
                            Property::Group => push_nonempty(&mut out, validate("InitGroup", statement, identifiers)),
                            Property::Tele => {
                                if let Instruction::PartialTele(id) = validate("InitTele", statement, identifiers) {
                                    // Inner "scope" of tele 
                                    let (inner, inner_last) = parse_inner(&statements[i + 1..], Some(id), identifiers);
                                    // Advance outer parse() call past the last instruction of the
                                    // inner call
                                    iter.nth(inner_last);
                                    push_nonempty(&mut out, Instruction::Tele(Tele {
                                        identifier: id,
                                        instructions: inner
                                    }));
                                }
                            },
                            // Define a static variable
                            Property::Float => {
                                if let Instruction::PartialFloat(id) = validate("InitFloat", statement, identifiers) {
                                    if let Some((next_i, next)) = iter.next() {
                                        if let Some(Target::Property(Property::You)) = next.action_target {
                                            let mut valid = false;
                                            let init = validate("FloatYou", next, identifiers);
                                            if let Instruction::Simple(Simple::InitYou(next_id, _float)) = init {
                                                if next_id == id {
                                                    valid = true;
                                                }
                                            }
                                            else if let Instruction::Complex(c) = init.to_owned() {
                                                let s = c.instruction;
                                                if let Simple::InitYou(next_id, _float) = s {
                                                    if next_id == id {
                                                        valid = true;
                                                    }
                                                }
                                            }
                                            if valid {
                                                push_nonempty(&mut out, init);
                                            }
                                            else {
                                                throw_error_str(
                                                    ErrorType::InstructionValidationError, 
                                                    "The subjects of FLOAT and YOU must match."
                                                );
                                            }
                                        }
                                        else if let Some(Target::Property(Property::You2)) = next.action_target {
                                            let mut valid = false;
                                            let init = validate("FloatYou2", next, identifiers);
                                            if let Instruction::Simple(Simple::InitYou2(next_id, _float)) = init {
                                                if next_id == id {
                                                    valid = true;
                                                }
                                            }
                                            else if let Instruction::Complex(c) = init.to_owned() {
                                                let s = c.instruction;
                                                if let Simple::InitYou2(next_id, _float) = s {
                                                    if next_id == id {
                                                        valid = true;
                                                    }
                                                }
                                            }
                                            if valid {
                                                push_nonempty(&mut out, init);
                                            }
                                            else {
                                                throw_error_str(
                                                    ErrorType::InstructionValidationError, 
                                                    "The subjects of FLOAT and YOU2 must match."
                                                );
                                            }
                                        }
                                        else if let Some(Target::Property(Property::Group)) = next.action_target {
                                            let init = validate("FloatGroup", next, identifiers);
                                            let mut valid = false;
                                            if let Instruction::Simple(Simple::InitGroup(next_id, _float)) = init {
                                                if next_id == id {
                                                    valid = true
                                                }
                                            }
                                            else if let Instruction::Complex(c) = init.to_owned() {
                                                let s = c.instruction;
                                                if let Simple::InitGroup(next_id, _float) = s {
                                                    if next_id == id {
                                                        valid = true;
                                                    }
                                                }
                                            }
                                            if valid {
                                                push_nonempty(&mut out, init);
                                            }
                                            else {
                                                throw_error_str(
                                                    ErrorType::InstructionValidationError, 
                                                    "The subjects of FLOAT and GROUP must match."
                                                );
                                            }
                                        }
                                        else if let Some(Target::Noun(Noun::Level)) = next.action_target {
                                            if let Instruction::PartialLevel(next_id) = validate("InitLevel", next, identifiers) {
                                                if id == next_id {

                                                    // Instructions in inner scope
                                                    let (inner, inner_last) = parse_inner(&statements[next_i + 1..], Some(next_id), identifiers);
                                                    // Advance outer parse() call past the last instruction of the
                                                    // inner call
                                                    iter.nth(inner_last);
                                                    // Parse inner loop for function arguments
                                                    let mut inner_loop = inner.iter();
                                                    let mut args = Vec::new();
                                                    while let Some(Instruction::Simple(Simple::HasValue(source, target))) = inner_loop.next() {
                                                        if *source == next_id {
                                                            args.push(*target);
                                                        }
                                                        else {
                                                            break;
                                                        }
                                                    }
                                                    if inner.len() == 0 {
                                                        push_nonempty(&mut out, Instruction::Level(Level {
                                                            identifier: id,
                                                            float: true,
                                                            arguments: args,
                                                            instructions: vec![]
                                                        }));
                                                    }
                                                    else {
                                                        let count = args.len();
                                                        push_nonempty(&mut out, Instruction::Level(Level {
                                                            identifier: id,
                                                            float: true,
                                                            arguments: args,
                                                            instructions: inner[count..].to_vec()
                                                        }));
                                                    }
                                                }
                                                else {
                                                    throw_error_str(
                                                        ErrorType::InstructionValidationError, 
                                                        "The subjects of FLOAT and LEVEL must match."
                                                    );
                                                }
                                            }
                                        }
                                        else if let Some(Target::Noun(Noun::Image)) = next.action_target {
                                            if let Instruction::PartialImage(next_id) = validate("InitImage", next, identifiers) {
                                                if id == next_id {
                                                    // Inner scope of class
                                                    let (inner, inner_last) = parse_inner(&statements[next_i + 1..], Some(next_id), identifiers);
                                                    // Advance outer parse() call past the last instruction of the
                                                    // inner call
                                                    iter.nth(inner_last);
                                                    // Parse inner scope for attributes and functions
                                                    // Any other instructions will panic.
                                                    let mut inner_loop = inner.iter();
                                                    let mut args = Vec::new();
                                                    let mut constructor = None;
                                                    while let Some(instr) = inner_loop.next() {
                                                        if let Instruction::Simple(Simple::HasValue(source, target)) = instr {
                                                            if *source == next_id {
                                                                args.push(*target);
                                                            }
                                                        }
                                                        else if let Instruction::Level(level) = instr {
                                                            if level.identifier == next_id {
                                                                if level.arguments.len() >= 1 {
                                                                    constructor = Some(level);
                                                                    break;
                                                                }
                                                                else {
                                                                    throw_error_str(
                                                                        ErrorType::InstructionValidationError, 
                                                                        "Class method must take at least one argument"
                                                                    )
                                                                }
                                                            }
                                                        }
                                                        else {
                                                            throw_error_str(
                                                                ErrorType::InstructionParserError, 
                                                                "IMAGE body may only contain attributes or function definitions"
                                                            )
                                                        }
                                                    }
                                                    if let Some(cons) = constructor {
                                                        push_nonempty(&mut out, Instruction::Image(Image {
                                                            identifier: next_id,
                                                            float: true,
                                                            attributes: args,
                                                            constructor: cons.to_owned()
                                                        }));
                                                    }
                                                    else {
                                                        throw_error_str(
                                                            ErrorType::InstructionValidationError, 
                                                            "IMAGE objects must define a constructor"
                                                        )
                                                    }
                                                }
                                                else {
                                                    throw_error_str(
                                                        ErrorType::InstructionValidationError, 
                                                        "The subjects of FLOAT and IMAGE must match"
                                                    )
                                                }
                                            }
                                        }
                                        else {
                                            throw_error(
                                                ErrorType::InstructionValidationError, 
                                                format!("[{0}] IS FLOAT must be followed by [{0}] IS YOU, YOU2, GROUP, LEVEL or IMAGE", id),
                                                Some((&[id], identifiers))
                                            );
                                        }
                                    }
                                }
                            },
                            // Type-indifferent instructions
                            Property::Text => push_nonempty(&mut out, validate("IsText", statement, identifiers)),
                            Property::Word => push_nonempty(&mut out, validate("IsWord", statement, identifiers)),
                            Property::Win => push_nonempty(&mut out, validate("IsWin", statement, identifiers)),
                            Property::Defeat => push_nonempty(&mut out, validate("IsDefeat", statement, identifiers)),
                            // YOU instructions
                            Property::Move => push_nonempty(&mut out, validate("YouMove", statement, identifiers)),
                            Property::Turn => push_nonempty(&mut out, validate("YouTurn", statement, identifiers)),
                            Property::Fall => push_nonempty(&mut out, validate("YouFall", statement, identifiers)),
                            Property::More => push_nonempty(&mut out, validate("YouMore", statement, identifiers)),
                            Property::Right => push_nonempty(&mut out, validate("YouRight", statement, identifiers)),
                            Property::Up => push_nonempty(&mut out, validate("YouUp", statement, identifiers)),
                            Property::Left => push_nonempty(&mut out, validate("YouLeft", statement, identifiers)),
                            Property::Down => push_nonempty(&mut out, validate("YouDown", statement, identifiers)),
                            // GROUP instructions
                            Property::Shift => push_nonempty(&mut out, validate("GroupShift", statement, identifiers)),
                            Property::Sink => push_nonempty(&mut out, validate("GroupSink", statement, identifiers)),
                            Property::Swap => push_nonempty(&mut out, validate("GroupSwap", statement, identifiers)),
                            // LEVEL instructions
                            Property::Power => push_nonempty(&mut out, validate("LevelPower", statement, identifiers)),
                        }
                    }
                    else if let Target::Noun(noun) = target {
                        if let Noun::Empty = noun {
                            push_nonempty(&mut out, validate("IsEmpty", statement, identifiers));
                        }
                        else if let Noun::Level = noun {
                            if let Instruction::PartialLevel(id) = validate("InitLevel", statement, identifiers) {
                                // Instructions in inner scope
                                let (inner, inner_last) = parse_inner(&statements[i + 1..], Some(id), identifiers);
                                // Advance outer parse() call past the last instruction of the
                                // inner call
                                iter.nth(inner_last);
                                // Parse inner loop for function arguments
                                let mut inner_loop = inner.iter();
                                let mut args = Vec::new();
                                while let Some(Instruction::Simple(Simple::HasValue(source, target))) = inner_loop.next() {
                                    if *source == id {
                                        args.push(*target);
                                    }
                                    else {
                                        break;
                                    }
                                }
                                if inner.len() == 0 {
                                    push_nonempty(&mut out, Instruction::Level(Level {
                                        identifier: id,
                                        float: false,
                                        arguments: args,
                                        instructions: vec![]
                                    }));
                                }
                                else {
                                    let count = args.len();
                                    push_nonempty(&mut out, Instruction::Level(Level {
                                        identifier: id,
                                        float: false,
                                        arguments: args,
                                        instructions: inner[count..].to_vec()
                                    }));
                                }
                            }
                        }
                        else if let Noun::Image = noun {
                            if let Instruction::PartialImage(id) = validate("InitImage", statement, identifiers) {
                                // Inner scope of class
                                let (inner, inner_last) = parse_inner(&statements[i + 1..], Some(id), identifiers);
                                // Advance outer parse() call past the last instruction of the
                                // inner call
                                iter.nth(inner_last);
                                // Parse inner scope for attributes and functions
                                // Any other instructions will panic.
                                let mut inner_loop = inner.iter();
                                let mut args = Vec::new();
                                let mut constructor = None;
                                while let Some(instr) = inner_loop.next() {
                                    if let Instruction::Simple(Simple::HasValue(source, target)) = instr {
                                        if *source == id {
                                            args.push(*target);
                                        }
                                    }
                                    else if let Instruction::Level(level) = instr {
                                        if level.identifier == id {
                                            if level.arguments.len() >= 1 {
                                                constructor = Some(level.to_owned());
                                                break;
                                            }
                                            else {
                                                throw_error_str(
                                                    ErrorType::InstructionValidationError, 
                                                    "Class method must take at least one argument"
                                                )
                                            }
                                        }
                                    }
                                    else {
                                        throw_error_str(
                                            ErrorType::InstructionParserError, 
                                            "IMAGE body may only contain attributes or function definitions"
                                        )
                                    }
                                }
                                if let Some(cons) = constructor {
                                    push_nonempty(&mut out, Instruction::Image(Image {
                                        identifier: id,
                                        float: false,
                                        attributes: args,
                                        constructor: cons
                                    }));
                                }
                                else {
                                    throw_error_str(
                                        ErrorType::InstructionValidationError, 
                                        "IMAGE objects must define a constructor"
                                    )
                                }
                            }
                        }
                        else {
                            push_nonempty(&mut out, validate("IsValue", statement, identifiers));
                        }
                    }
                }
                else if let Some(_) = &statement.action_targets {
                    push_nonempty(&mut out, validate("YouSum", statement, identifiers));
                }
            },
            Verb::Has => {
                if let Some(target) = statement.action_target {
                    if let Target::Noun(_) = target {
                        push_nonempty(&mut out, validate("HasValue", statement, identifiers));
                    }
                }
            },
            Verb::Make => {
                if let Some(target) = statement.action_target {
                    if let Target::Noun(_) = target {
                        push_nonempty(&mut out, validate("MakeValue", statement, identifiers));
                    }
                }
            },
            Verb::Follow => {
                if let Some(target) = statement.action_target {
                    if let Target::Noun(_) = target {
                        push_nonempty(&mut out, validate("FollowAttribute", statement, identifiers));
                    }
                }
            },
            Verb::Mimic => {
                if let Some(target) = statement.action_target {
                    if let Target::Noun(_) = target {
                        push_nonempty(&mut out, validate("MimicReference", statement, identifiers));
                    }
                }
            },
            Verb::Fear => {
                if let Some(target) = statement.action_target {
                    if let Target::Noun(_) = target {
                        push_nonempty(&mut out, validate("FearTele", statement, identifiers));
                    }
                }
            },
            Verb::Eat => {
                if let Some(target) = statement.action_target {
                    if let Target::Noun(_) = target {
                        push_nonempty(&mut out, validate("EatValue", statement, identifiers));
                    }
                }
            },
            _ => {
                throw_error(
                    ErrorType::InstructionParserError, 
                    format!("Invalid verb `{:?}` provided to instruction parser", action_type),
                    None
                );
            }
        }
    }
    (out, last)
}

