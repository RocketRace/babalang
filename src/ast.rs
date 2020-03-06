use crate::instruction::{Instruction, validate};
use crate::statement::{Statement, Target};
use crate::token::{VerbToken, PropertyToken};

pub fn parse(statements: &[Statement]) -> Vec<Instruction> {
    let mut out = Vec::new();
    for statement in statements {
        let action_type = statement.action_type;
        match action_type {
            VerbToken::Is => {
                if let Some(target) = statement.action_target {
                    if let Target::Property(prop) = target {
                        match prop {
                            PropertyToken::You => {
                                out.push(validate("InitYou", statement));
                            },
                            PropertyToken::Move => {
                                out.push(validate("YouMove", statement));
                            },
                            PropertyToken::Text => {
                                out.push(validate("Text", statement));
                            },
                            PropertyToken::Fall => {
                                out.push(validate("YouFall", statement));
                            },
                            _ => {
                                panic!("Invalid instruction!")
                            }
                        }
                    }
                    else if let Target::Noun(noun) = target {

                    }
                }
            },
            _ => {
                panic!("Unimplemented!");
            }
        }
    }

    out
}

