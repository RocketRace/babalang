use crate::error_handler::{throw_error, throw_error_str, ErrorType};
use crate::statement::{Statement, Target};
use crate::token::{Noun, Conditional, Prefix};

/// Describes an instruction without conditions.
#[derive(Debug, Copy, Clone)]
pub enum Simple<'a> {
    // init
    InitYou(usize),
    InitGroup(usize),
    // any
    Text(usize),
    IsValue(usize, usize),
    // you/all
    IsSum(usize, &'a [usize]),
    Move(usize, bool),
    Fall(usize, bool),
    AllMove,
    AllFall,
    Right(usize, bool),
    Up(usize, bool),
    Left(usize, bool),
    Down(usize, bool),
    AllRight,
    AllUp,
    AllLeft,
    AllDown,
    // group
    Shift(usize, bool),
    Push(usize),
}

/// Describes an instruction with some conditions.
#[derive(Debug, Copy, Clone)]
pub struct Complex<'a> {
    conditions: Option<Conditions<'a>>,
    prefix: Option<Prefixes>,
    instruction: Simple<'a>
}

/// Descrives the targeted conditions for a complex instruction.
#[derive(Debug, Copy, Clone)]
pub struct Conditions<'a> {
    cond_type: Conditional,
    targets: &'a [Target],
    sign: bool
}

/// Describes the non-targeted (unary) conditions for a complex instruction.
#[derive(Debug, Copy, Clone)]
pub struct Prefixes {
    prefix: Prefix,
    sign: bool,
}

/// Descibes a TELE instruction (i.e. a loop instruction).
#[derive(Debug, Clone)]
pub struct Tele<'a> {
    pub instructions: Vec<Instruction<'a>>,
    pub identifier: usize
}

/// Describes an instruction.
#[derive(Debug, Clone)]
pub enum Instruction<'a> {
    NoOp,
    Simple(Simple<'a>),
    Complex(Complex<'a>),
    PartialTele(usize),
    Tele(Tele<'a>), // Loop
    Level, // Function definition
    Image // Class definition
}

/// Validates an instruction. Throws an InstructionValidationError if the attempted
/// instruction can't be constructed from the statement.
pub fn validate<'a>(instruction_type: &str, statement: &'a Statement) -> Instruction<'a> {
    let mut instr = Instruction::NoOp;
    match instruction_type {
        "InitYou" => instr = generic_init(statement, "YOU", &Simple::InitYou),
        "InitGroup" => instr = generic_init(statement, "Group", &Simple::InitGroup),
        "InitTele" => {
            let conds = conditions(statement);
            if let Noun::Identifier(id) = statement.subject {
                if let (None, None) = conds {
                    if !statement.action_sign {
                        instr = Instruction::PartialTele(id);
                    }
                    else {
                        // NOT TELE is a no-op
                        instr = Instruction::NoOp
                    }
                }
                else {
                    throw_error_str(
                        ErrorType::InstructionValidationError, 
                        "IS TELE can not be defined with conditions."
                    )
                }
            }
            else {
                throw_error(
                    ErrorType::InstructionValidationError, 
                    format!("Cannot initialize {:?} as TELE.", statement.subject)
                );
            }
        },
        "Text" => instr = generic_any(statement, "TEXT", &Simple::Text),
        "IsValue" => instr = generic_verb(statement, "IS", &Simple::IsValue),
        "YouMove" => instr = generic_you(statement, "MOVE", &Simple::Move, Simple::AllMove),
        "YouFall" => instr = generic_you(statement, "FALL", &Simple::Fall, Simple::AllFall),
        "YouRight" => instr = generic_you(statement, "RIGHT", &Simple::Right, Simple::AllRight),
        "YouUp" => instr = generic_you(statement, "UP", &Simple::Up, Simple::AllUp),
        "YouLeft" => instr = generic_you(statement, "LEFT", &Simple::Left, Simple::AllLeft),
        "YouDown" => instr = generic_you(statement, "DOWN", &Simple::Down, Simple::AllDown),
        _ => {
            throw_error_str(
                ErrorType::InstructionValidationError, 
                &format!("Attempted to parse invalid instruction {}", instruction_type)
            );
        }
    }
    instr
}

/// Retrieves the conditions associated with a statement.
/// 
/// Returns a 2-tuple containing the Complex condition and the Prefix condition.
/// These will be None if the associated condition does not exist.
/// 
/// # Examples
/// 
/// * `BABA IS YOU` -> (None, None)
/// 
/// * `LONELY BABA IS YOU` -> (None, Some(<Lonely>))
/// 
/// * `BABA ON KEKE IS YOU` -> (Some(<On Keke>), None)
/// 
/// * `LONELY BABA NEAR KEKE IS YOU` -> (Some(<Near Keke>), Some(<Lonely>))
/// 
pub fn conditions(statement: &Statement) -> (Option<Conditions>, Option<Prefixes>) {
    if let Some(cond) = statement.cond_type {
        if let Some(pref) = statement.prefix {
            // Prefix and condition
            (
                Some(Conditions {
                    cond_type: cond,
                    sign: statement.cond_sign.unwrap(),
                    targets: &statement.cond_targets
                }),
                Some(Prefixes {
                    prefix: pref,
                    sign: statement.prefix_sign.unwrap()
                })
            )
        }
        else {
            // Only condition
            (
                Some(Conditions {
                    cond_type: cond,
                    sign: statement.cond_sign.unwrap(),
                    targets: &statement.cond_targets
                }),
                None
            )
        }
    }
    else if let Some(pref) = statement.prefix {
        // Only prefix
        (
            None,
            Some(Prefixes {
                prefix: pref,
                sign: statement.prefix_sign.unwrap()
            })
        )
    }
    else {
        // No conditions
        (None, None)
    }
}

/// Merges a simple instruction with conditions into a Complex instruction.
fn merge<'a>(
    simple: Simple<'a>,
    conds: (Option<Conditions<'a>>, Option<Prefixes>),
) -> Instruction<'a> {
    let (cond, prefix) = conds;
    if let (None, None) = (cond, prefix) {
        Instruction::Simple(simple)
    }
    else {
        Instruction::Complex(Complex {
            conditions: cond,
            prefix: prefix,
            instruction: simple
        })
    }
}

/// Returns a reversible YOU instruction with default parameters.
/// 
/// Allows for the use of ALL, as well as NOT to reverse instructions.
fn generic_you<'a>(
    statement: &'a Statement,
    target: &str,
    simple_factory: &dyn Fn(usize, bool) -> Simple<'a>,
    all: Simple<'a>
) -> Instruction<'a> {
    let conds = conditions(statement);
    if let Noun::Identifier(id) = statement.subject {
        let simple = simple_factory(id, statement.action_sign); 
        merge(simple, conds)
    }
    else if let Noun::All = statement.subject {
        let simple = all;
        merge(simple, conds)
    }
    else {
        throw_error(
            ErrorType::InstructionValidationError, 
            format!("Cannot apply {} to {:?}.", target, statement.subject)
        );
        Instruction::NoOp
    }
}

/// Returns a reversible YOU instruction with default parameters.
/// 
/// Allows for the use of ALL, as well as NOT to reverse instructions.
fn generic_init<'a>(
    statement: &'a Statement,
    target: &str,
    simple_factory: &dyn Fn(usize) -> Simple<'a>,
) -> Instruction<'a> {
    let conds = conditions(statement);
    if let Noun::Identifier(id) = statement.subject {
        if let (None, None) = conds {
            if !statement.action_sign {
                Instruction::Simple(simple_factory(id))
            }
            else {
                // NOT [type] is a no-op
                Instruction::NoOp
            }
        }
        else {
            throw_error(
                ErrorType::InstructionValidationError, 
                format!("IS {} can not be defined with conditions.", target)
            );
            Instruction::NoOp

        }
    }
    else {
        throw_error(
            ErrorType::InstructionValidationError, 
            format!("Cannot initialize {:?} as {}.", statement.subject, target)
        );
        Instruction::NoOp
    }
}


/// Returns a nonreversible YOU/GROUP instruction with default parameters.
/// 
/// Does NOT allow for the use of ALL, or NOT.
fn generic_any<'a>(
    statement: &'a Statement,
    target: &str,
    simple_factory: &dyn Fn(usize) -> Simple<'a>,
) -> Instruction<'a> {
    let conds = conditions(statement);
    if let Noun::Identifier(id) = statement.subject {
        let simple = simple_factory(id); 
        merge(simple, conds)
    }
    else {
        throw_error(
            ErrorType::InstructionValidationError, 
            format!("Cannot apply {} to {:?}.", target, statement.subject)
        );
        Instruction::NoOp
    }
}

/// Returns a generic NOUN VERB NOUN instruction.
/// 
/// Does not allow for the use of NOT.
fn generic_verb<'a>(
    statement: &'a Statement,
    target: &str,
    simple_factory: &dyn Fn(usize, usize) -> Simple<'a>,
) -> Instruction<'a> {
    let conds = conditions(statement);
    if let Noun::Identifier(id) = statement.subject {
        if let Some(Target::Noun(Noun::Identifier(source))) = statement.action_target {
            let simple = simple_factory(id, source); 
            merge(simple, conds)
        }
        else {
            Instruction::NoOp
        }
    }
    else {
        throw_error(
            ErrorType::InstructionValidationError, 
            format!("Cannot make {:?} {} any noun.", statement.subject, target)
        );
        Instruction::NoOp
    }
}