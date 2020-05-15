use crate::error_handler::{throw_error, throw_error_str, ErrorType};
use crate::statement::{Statement, Target};
use crate::token::{Noun, Conditional, Prefix};

use std::collections::HashMap;

/// Describes an instruction without conditions.
#[derive(Debug, Clone, PartialEq)]
pub enum Simple {
    // init
    InitYou(usize, bool),
    InitYou2(usize, bool),
    InitGroup(usize, bool),
    // any
    Win(usize),
    Defeat(usize),
    Sleep(usize),
    Text(usize),
    Word(usize),
    IsValue(usize, usize, bool),
    MimicReference(usize, usize),
    IsEmpty(usize),
    // you
    IsSum(usize, Vec<Noun>, Vec<bool>),
    Move(usize, bool),
    Turn(usize, bool),
    Fall(usize, bool),
    More(usize, bool),
    Right(usize, bool),
    Up(usize, bool),
    Left(usize, bool),
    Down(usize, bool),
    Chill(usize, bool),
    // all (subset of you)
    AllMove(bool),
    AllTurn(bool),
    AllFall(bool),
    AllMore(bool),
    AllRight(bool),
    AllUp(bool),
    AllLeft(bool),
    AllDown(bool),
    AllChill(bool),
    // group
    Shift(usize, bool),
    Sink(usize),
    Swap(usize),
    // group / level
    HasValue(usize, usize),
    MakeValue(usize, usize),
    // level
    Power(usize, bool),
    // tele
    FearTele(usize, usize),
    // image
    FollowAttribute(usize, usize),
    EatValue(usize, usize),
}

/// Describes an instruction with some conditions.
/// Both `conditions` and `prefix` should typically not be None.
#[derive(Debug, Clone, PartialEq)]
pub struct Complex {
    pub conditions: Option<Conditions>,
    pub prefix: Option<Prefixes>,
    pub instruction: Simple
}

/// Descrives the targeted conditions for a complex instruction.
#[derive(Debug, Clone, PartialEq)]
pub struct Conditions{
    pub cond_type: Conditional,
    pub targets: Vec<Target>,
    pub sign: bool
}

/// Describes the non-targeted (unary) conditions for a complex instruction.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Prefixes {
    pub prefix: Prefix,
    pub sign: bool,
}

/// Descibes a TELE instruction (i.e. a loop instruction).
#[derive(Debug, Clone, PartialEq)]
pub struct Tele {
    pub identifier: usize,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Level {
    pub float: bool,
    pub identifier: usize,
    pub arguments: Vec<usize>,
    pub instructions: Vec<Instruction>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub float: bool,
    pub identifier: usize,
    pub attributes: Vec<usize>,
    pub constructor: Level
}

/// Describes an instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    NoOp,
    Simple(Simple),
    Complex(Complex),
    PartialTele(usize),
    Tele(Tele), // Loops
    PartialLevel(usize),
    Level(Level), // Function definition
    PartialImage(usize),
    Image(Image), // Class definition
    PartialFloat(usize) // Static variables
}

/// Validates an instruction. Throws an InstructionValidationError if the attempted
/// instruction can't be constructed from the statement.
pub fn validate<'a>(
    instruction_type: &str, 
    statement: &'a Statement,
    identifiers: &HashMap<usize, String>
) -> Instruction {
    let mut instr = Instruction::NoOp;
    match instruction_type {
        "InitYou" => instr = generic_init(statement, "YOU", false, &Simple::InitYou),
        "InitYou2" => instr = generic_init(statement, "YOU2", false, &Simple::InitYou2),
        "InitGroup" => instr = generic_init(statement, "GROUP", false, &Simple::InitGroup),
        "InitTele" => instr = generic_partial(statement, "TELE", &Instruction::PartialTele),
        "InitLevel" => instr = generic_partial(statement, "LEVEL", &Instruction::PartialLevel),
        "InitImage" => instr = generic_partial(statement, "IMAGE", &Instruction::PartialImage),
        "InitFloat" => instr = generic_partial(statement, "FLOAT", &Instruction::PartialFloat),
        "FloatYou" => instr = generic_init(statement, "YOU", true, &Simple::InitYou),
        "FloatYou2" => instr = generic_init(statement, "YOU2", true, &Simple::InitYou2),
        "FloatGroup" => instr = generic_init(statement, "GROUP", true, &Simple::InitGroup),
        "IsText" => instr = generic_any(statement, "TEXT", &Simple::Text),
        "IsWord" => instr = generic_any(statement, "WORD", &Simple::Word),
        "IsWin" => instr = generic_any(statement, "WIN", &Simple::Win),
        "IsDefeat" => instr = generic_any(statement, "DEFEAT", &Simple::Defeat),
        "IsSleep" => instr = generic_any(statement, "SLEEP", &Simple::Sleep),
        "IsEmpty" => instr = generic_any(statement, "EMPTY", &Simple::IsEmpty),
        "IsValue" => {
            let conds = conditions(statement);
            if let Noun::Identifier(id) = statement.subject {
                if let Some(Target::Noun(Noun::Identifier(source))) = statement.action_target {
                    let simple = Simple::IsValue(id, source, statement.action_sign); 
                    instr = merge(simple, conds);
                }
            }
            else {
                if let Some(noun) = statement.action_target {
                    if let Target::Noun(Noun::Identifier(other_id)) = noun {
                        throw_error(
                            ErrorType::InstructionValidationError, 
                            format!("Cannot make {:?} IS Identifier({})", statement.subject, other_id),
                            Some((&[other_id], identifiers))
                        );
                    }
                    else {
                        throw_error(
                            ErrorType::InstructionValidationError, 
                            format!("Cannot make {:?} IS {:?}", statement.subject, noun),
                            None
                        );
                    }
                }
                else {
                    throw_error(
                        ErrorType::InstructionValidationError, 
                        format!("Cannot make {:?} IS any noun", statement.subject),
                        None
                    );
                }
            }
        }
        "HasValue" => instr = generic_verb(statement, "HAS", &Simple::HasValue),
        "MakeValue" => instr = generic_verb(statement, "MAKE", &Simple::MakeValue),
        "FollowAttribute" => instr = generic_verb(statement, "FOLLOW", &Simple::FollowAttribute),
        "EatValue" => instr = generic_verb(statement, "EAT", &Simple::EatValue),
        "MimicReference" => {
            let conds = conditions(statement);
            if let Noun::Identifier(id) = statement.subject {
                if let Some(Target::Noun(Noun::Identifier(source))) = statement.action_target {
                    let simple = Simple::MimicReference(id, source); 
                    instr = merge(simple, conds);
                }
                else {
                    throw_error(
                        ErrorType::InstructionValidationError, 
                        format!("Cannot make {} MIMIC {:?}", id, statement.action_target),
                        Some((&[id], identifiers))
                    );
                }
            }
            else {
                if let Some(noun) = statement.action_target {
                    if let Target::Noun(Noun::Identifier(other_id)) = noun {
                        throw_error(
                            ErrorType::InstructionValidationError, 
                            format!("Cannot make {:?} MIMIC Identifier({})", statement.subject, other_id),
                            Some((&[other_id], identifiers))
                        );
                    }
                    else {
                        throw_error(
                            ErrorType::InstructionValidationError, 
                            format!("Cannot make {:?} MIMIC {:?}", statement.subject, noun),
                            None
                        );
                    }
                }
                else {
                    throw_error(
                        ErrorType::InstructionValidationError, 
                        format!("Cannot make {:?} MIMIC any noun", statement.subject),
                        None
                    );
                }
            }
        },
        "FearTele" => instr = generic_verb(statement, "FEAR", &Simple::FearTele),
        "YouMove" => instr = generic_you(statement, "MOVE", &Simple::Move, &Simple::AllMove),
        "YouTurn" => instr = generic_you(statement, "TURN", &Simple::Turn, &Simple::AllTurn),
        "YouFall" => instr = generic_you(statement, "FALL", &Simple::Fall, &Simple::AllFall),
        "YouMore" => instr = generic_you(statement, "MORE", &Simple::More, &Simple::AllMore),
        "YouRight" => instr = generic_you(statement, "RIGHT", &Simple::Right, &Simple::AllRight),
        "YouUp" => instr = generic_you(statement, "UP", &Simple::Up, &Simple::AllUp),
        "YouLeft" => instr = generic_you(statement, "LEFT", &Simple::Left, &Simple::AllLeft),
        "YouDown" => instr = generic_you(statement, "DOWN", &Simple::Down, &Simple::AllDown),
        "YouChill" => instr = generic_you(statement, "CHILL", &Simple::Chill, &Simple::AllChill),
        "YouSum" => {
            let conds = conditions(statement);
            instr = if let Noun::Identifier(id) = statement.subject {
                if let (Some(targets), Some(signs)) = (statement.action_targets.clone(), statement.action_signs.clone()) {
                    let simple = Simple::IsSum(id, targets, signs); 
                    merge(simple, conds)
                }
                else {
                    Instruction::NoOp
                }
            }
            else {
                throw_error(
                    ErrorType::InstructionValidationError, 
                    format!("Cannot set {:?} to sum of objects", statement.subject),
                    None
                );
                Instruction::NoOp
            }
        }
        "GroupShift" => instr = generic_not(statement, "SHIFT", &Simple::Shift),
        "GroupSink" => instr = generic_any(statement, "SINK", &Simple::Sink),
        "GroupSwap" => instr = generic_any(statement, "SWAP", &Simple::Swap),
        // Power is generic_init, 
        "LevelPower" => instr = generic_init(statement, "POWER", false, &Simple::Power),
        "FloatPower" => instr = generic_init(statement, "POWER", true,  &Simple::Power),
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
                    targets: statement.cond_targets.to_owned()
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
                    targets: statement.cond_targets.to_owned()
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
    simple: Simple,
    conds: (Option<Conditions>, Option<Prefixes>),
) -> Instruction {
    let (cond, prefix) = conds;
    match (cond, prefix) {
        (None, None) => Instruction::Simple(simple),
        (Some(c), p) => Instruction::Complex(Complex {
            conditions: Some(c),
            prefix: p,
            instruction: simple
        }),
        (None, p) => Instruction::Complex(Complex {
            conditions: None,
            prefix: p,
            instruction: simple
        }),
    }
}

/// Returns a reversible YOU instruction with default parameters.
/// 
/// Allows for the use of ALL, as well as NOT to reverse instructions.
fn generic_you<'a>(
    statement: &'a Statement,
    target: &str,
    simple_factory: &dyn Fn(usize, bool) -> Simple,
    all_factory: &dyn Fn(bool) -> Simple
) -> Instruction {
    let conds = conditions(statement);
    if let Noun::Identifier(id) = statement.subject {
        let simple = simple_factory(id, statement.action_sign); 
        merge(simple, conds)
    }
    else if let Noun::All = statement.subject {
        let simple = all_factory(statement.action_sign);
        merge(simple, conds)
    }
    else {
        throw_error(
            ErrorType::InstructionValidationError, 
            format!("Cannot apply {} to {:?}", target, statement.subject),
            None
        );
        Instruction::NoOp
    }
}

/// Returns a reversible GROUP instruction with default parameters.
/// 
/// Allows for the use NOT to reverse instructions.
fn generic_not<'a>(
    statement: &'a Statement,
    target: &str,
    simple_factory: &dyn Fn(usize, bool) -> Simple,
) -> Instruction {
    let conds = conditions(statement);
    if let Noun::Identifier(id) = statement.subject {
        let simple = simple_factory(id, statement.action_sign); 
        merge(simple, conds)
    }
    else {
        throw_error(
            ErrorType::InstructionValidationError, 
            format!("Cannot apply {} to {:?}", target, statement.subject),
            None
        );
        Instruction::NoOp
    }
}

/// Returns an INIT instruction with default parameters.
/// 
/// Does not allow for conditionals. NOT returns a no-op.
fn generic_init<'a>(
    statement: &'a Statement,
    target: &str,
    float: bool,
    simple_factory: &dyn Fn(usize, bool) -> Simple,
) -> Instruction {
    let conds = conditions(statement);
    if let Noun::Identifier(id) = statement.subject {
        if target == "POWER" { // Hacky way to allow for FLOATing POWER
            Instruction::Simple(simple_factory(id, float))
        }
        else if let (None, None) = conds {
            if !statement.action_sign {
                Instruction::Simple(simple_factory(id, float))
            }
            else {
                // NOT [type] is a no-op
                Instruction::NoOp
            }
        }
        else {
            throw_error(
                ErrorType::InstructionValidationError, 
                format!("IS {} cannot be defined with conditions", target),
                None
            );
            Instruction::NoOp

        }
    }
    else {
        throw_error(
            ErrorType::InstructionValidationError, 
            format!("Cannot initialize {:?} as {}", statement.subject, target),
            None
        );
        Instruction::NoOp
    }
}

/// Returns an INIT instruction with default parameters.
/// 
/// Does not allow for conditionals. NOT returns a no-op.
fn generic_partial<'a>(
    statement: &'a Statement,
    target: &str,
    partial_factory: &dyn Fn(usize) -> Instruction,
) -> Instruction {
    let conds = conditions(statement);
    if let Noun::Identifier(id) = statement.subject {
        if let (None, None) = conds {
            if !statement.action_sign {
                partial_factory(id)
            }
            else {
                // NOT YOU/GROUP/TELE is a no-op
                Instruction::NoOp
            }
        }
        else {
            throw_error(
                ErrorType::InstructionValidationError, 
                format!("IS {} cannot be called with conditions", target),
                None
            );
            Instruction::NoOp
        }
    }
    else {
        throw_error(
            ErrorType::InstructionValidationError, 
            format!("Cannot initialize {:?} as {}", statement.subject, target),
            None
        );
        Instruction::NoOp
    }
}


/// Returns a nonreversible YOU/GROUP instruction with default parameters.
/// 
/// Negation via NOT returns a no-op.
fn generic_any<'a>(
    statement: &'a Statement,
    target: &str,
    simple_factory: &dyn Fn(usize) -> Simple,
) -> Instruction {
    let conds = conditions(statement);
    if let Noun::Identifier(id) = statement.subject {
        if let false = statement.action_sign {
            let simple = simple_factory(id); 
            merge(simple, conds)
        }
        else {
            Instruction::NoOp
        }
    }
    else {
        throw_error(
            ErrorType::InstructionValidationError, 
            format!("Cannot apply {} to {:?}", target, statement.subject),
            None
        );
        Instruction::NoOp
    }
}

/// Returns a generic NOUN VERB NOUN instruction.
/// 
/// Negation via NOT returns a no-op.
fn generic_verb<'a>(
    statement: &'a Statement,
    target: &str,
    simple_factory: &dyn Fn(usize, usize) -> Simple,
) -> Instruction {
    let conds = conditions(statement);
    if let Noun::Identifier(id) = statement.subject {
        if let Some(Target::Noun(Noun::Identifier(source))) = statement.action_target {
            let simple = simple_factory(id, source); 
            merge(simple, conds)
        }
        else if let Some(Target::Noun(Noun::Empty)) = statement.action_target {
            let simple = simple_factory(id, 0); 
            merge(simple, conds)
        }
        else if let Some(Target::Noun(Noun::Level)) = statement.action_target {
            let simple = simple_factory(id, 1); 
            merge(simple, conds)
        }
        else {
            Instruction::NoOp
        }
    }
    else {
        throw_error(
            ErrorType::InstructionValidationError, 
            format!("Cannot make {:?} {} any noun", statement.subject, target),
            None
        );
        Instruction::NoOp
    }
}