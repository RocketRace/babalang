use crate::error_handler::{throw_error, throw_error_str, ErrorType};
use crate::statement::{Statement, Target};
use crate::token::{NounToken, ConditionalToken, PrefixToken};

#[derive(Debug, Copy, Clone)]
pub enum Simple {
    InitYou(usize),
    Move(usize, bool),
    Fall(usize, bool),
    Text(usize),
}

#[derive(Debug, Copy, Clone)]
pub struct Conditional<'a> {
    conditions: Option<Conditions<'a>>,
    prefix: Option<Prefix>,
    instruction: Simple
}

#[derive(Debug, Copy, Clone)]
pub struct Conditions<'a> {
    cond_type: ConditionalToken,
    targets: &'a [Target],
    sign: bool
}

#[derive(Debug, Copy, Clone)]
pub struct Prefix {
    prefix: PrefixToken,
    sign: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct Tele<'a> {
    instructions: &'a [Instruction<'a>],
    identifier: usize
}

#[derive(Debug, Copy, Clone)]
pub enum Instruction<'a> {
    NoOp,
    Simple(Simple),
    Conditional(Conditional<'a>),
    Tele(Tele<'a>), // Loop
    Level, // Function definition
    Image // Class definition
}

pub fn validate<'a>(instruction_type: &str, statement: &'a Statement) -> Instruction<'a> {
    let mut instr = Instruction::NoOp;
    match instruction_type {
        "InitYou" => {
            let (cond, prefix) = conditions(statement);
            if let NounToken::Identifier(id) = statement.subject {
                if let (None, None) = (cond, prefix) {
                    instr = Instruction::Simple(Simple::InitYou(id));
                }
                else {
                    throw_error_str(
                        ErrorType::InstructionParserError, 
                        "IS YOU can not be defined conditionally."
                    )
                }
            }
            else {
                throw_error(
                    ErrorType::InstructionParserError, 
                    format!("Cannot initialize {:?} as YOU.", statement.subject)
                );
            }
        },
        "YouMove" => {
            let conds = conditions(statement);
            if let NounToken::Identifier(id) = statement.subject {
                let simple = Simple::Move(id, statement.action_sign); 
                instr = merge(simple, conds);
            }
            else if let NounToken::All = statement.subject {
                instr = Instruction::NoOp;
                // TODO allow ALL IS MOVE
            }
            else {
                throw_error(
                    ErrorType::InstructionParserError, 
                    format!("Cannot apply MOVE to {:?}.", statement.subject)
                );
            }
        },
        "Text" => {
            let conds = conditions(statement);
            if let NounToken::Identifier(id) = statement.subject {
                let simple = Simple::Text(id); 
                instr = merge(simple, conds);
            }
            else if let NounToken::All = statement.subject {
                instr = Instruction::NoOp;
                // TODO allow ALL IS TEXT
            }
            else {
                throw_error(
                    ErrorType::InstructionParserError, 
                    format!("Cannot apply TEXT to {:?}.", statement.subject)
                );
            }
        },
        "YouFall" => {
            let conds = conditions(statement);
            if let NounToken::Identifier(id) = statement.subject {
                let simple = Simple::Fall(id, statement.action_sign); 
                instr = merge(simple, conds);
            }
            else if let NounToken::All = statement.subject {
                instr = Instruction::NoOp;
                // TODO allow ALL IS MOVE
            }
            else {
                throw_error(
                    ErrorType::InstructionParserError, 
                    format!("Cannot apply FALL to {:?}.", statement.subject)
                );
            }
        },
        _ => {
            throw_error_str(
                ErrorType::InstructionParserError, 
                &format!("Attempted to parse invalid instruction {}", instruction_type)
            );
        }
    }
    instr
}

/// Merges a simple instruction with conditionals into a conditional instruction.
fn merge<'a>(
    simple: Simple,
    conds: (Option<Conditions<'a>>, Option<Prefix>),
) -> Instruction<'a> {
    let (cond, prefix) = conds;
    if let (None, None) = (cond, prefix) {
        Instruction::Simple(simple)
    }
    else {
        Instruction::Conditional(Conditional {
            conditions: cond,
            prefix: prefix,
            instruction: simple
        })
    }
}

/// Retrieves the conditions associated with a statement.
/// 
/// Returns a 2-tuple containing the Conditional condition and the Prefix condition.
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
fn conditions(statement: &Statement) -> (Option<Conditions>, Option<Prefix>) {
    if let Some(cond) = statement.cond_type {
        if let Some(pref) = statement.prefix {
            // Prefix and condition
            (
                Some(Conditions {
                    cond_type: cond,
                    sign: statement.cond_sign.unwrap(),
                    targets: &statement.cond_targets
                }),
                Some(Prefix {
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
            Some(Prefix {
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