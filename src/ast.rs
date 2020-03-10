use crate::instruction::{Instruction, Tele, validate, conditions};
use crate::statement::{Statement, Target};
use crate::token::{Verb, Property, Noun};
use crate::error_handler::{throw_error, ErrorType, throw_error_str};

/// Parses a stream of statements into instructions.
/// 
/// If `scope` is given, will only parse until the given scope is exited
/// via `[id=scope] IS DONE`. Otherwise, will read until `ALL IS DONE` is 
/// encountered, or until the stream ends.
pub fn parse<'a>(statements: &'a [Statement], scope: Option<usize>) -> Vec<Instruction<'a>> {
    let mut out = Vec::new();
    let mut iter = statements.iter().enumerate();
    // This `for` loop is desugared to allow for elements to be skipped
    while let Some((i, statement)) = iter.next() {
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
                                                        return out;
                                                    }
                                                    _ => {
                                                        throw_error(
                                                            ErrorType::InstructionParserError, 
                                                            format!(
                                                                "Cannot exit out of {:?} when in global scope",
                                                                statement.subject 
                                                            )
                                                        )
                                                    }
                                                }
                                            },
                                            Noun::All => {
                                                if let None = scope {
                                                    // ALL IS DONE
                                                    if let false = statement.action_sign {
                                                        return out;
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
                                                    format!("Cannot exit out of {:?}", statement.subject)    
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
                            Property::You => out.push(validate("InitYou", statement)),
                            Property::Group => out.push(validate("InitGroup", statement)),
                            Property::Tele => {
                                if let Instruction::PartialTele(id) = validate("InitTele", statement) {
                                    let inner = parse(&statements[i + 1..], Some(id));
                                    iter.nth(inner.len());
                                    out.push(Instruction::Tele(Tele {
                                        identifier: id,
                                        instructions: inner
                                    }))
                                }
                            },
                            // Type-indifferent instructions
                            Property::Text => out.push(validate("Text", statement)),
                            // YOU instructions
                            Property::Move => out.push(validate("YouMove", statement)),
                            Property::Turn => out.push(validate("YouTurn", statement)),
                            Property::Fall => out.push(validate("YouFall", statement)),
                            Property::More => out.push(validate("YouMore", statement)),
                            Property::Right => out.push(validate("YouRight", statement)),
                            Property::Up => out.push(validate("YouUp", statement)),
                            Property::Left => out.push(validate("YouLeft", statement)),
                            Property::Down => out.push(validate("YouDown", statement)),
                            Property::Shift => out.push(validate("GroupShift", statement)),
                            Property::Push => out.push(validate("GroupPush", statement)),
                            Property::Sink => out.push(validate("GroupSink", statement)),
                            Property::Swap => out.push(validate("GroupSwap", statement)),
                            _ => {
                                throw_error(
                                    ErrorType::InstructionParserError, 
                                    format!("Invalid property `{:?}` provided to IS", prop)
                                );
                            }
                        }
                    }
                    else if let Target::Noun(_) = target {
                        out.push(validate("IsValue", statement));
                    }
                }
                else if let Some(_) = &statement.action_targets {
                    out.push(validate("YouSum", statement));
                }
            },
            Verb::Has => {
                if let Some(target) = statement.action_target {
                    if let Target::Noun(_) = target {
                        out.push(validate("HasValue", statement));
                    }
                }
            },
            Verb::Fear => {
                if let Some(target) = statement.action_target {
                    if let Target::Noun(_) = target {
                        out.push(validate("FearTele", statement));
                    }
                }
            },
            Verb::Eat => {
                if let Some(target) = statement.action_target {
                    if let Target::Noun(_) = target {
                        out.push(validate("EatTele", statement));
                    }
                }
            },
            _ => {
                throw_error(
                    ErrorType::InstructionParserError, 
                    format!("Invalid verb `{:?}` provided to instruction parser", action_type)
                );
            }
        }
    }

    out
}

