use crate::token::{NounToken, VerbToken, PropertyToken, ConditionalToken, LexToken};
use crate::error_handler::{ErrorType, throw_error};

/// The internal state of the finite state machine.
#[derive(Debug)]
enum ParserState {
    // Subject
    Blank,
    Subject,
    // Major conditional 
    ExpectsMajCond, MajCond, MajCondTarget, CondAnd, 
    MajCondFacing, MajCondFacingTarget, CondFacingAnd,
    // Minor conditional
    ExpectsMinCond, MinCond, MinCondFacing, MinCondTarget,
    // Major action: IS
    MajAct, MajActTarget, ActAnd, ExpectsMajActTarget,
    // Major action: other verbs
    MajIs, MajIsTarget, IsAnd, ExpectsMajIsTarget,
    // Minor actions
    ExpectsMinActTarget,
}

#[derive(Clone, Copy, Debug)]
pub enum Target {
    Noun(NounToken),
    Property(PropertyToken)
}

#[derive(Clone, Debug)]
pub struct Statement {
    subject: NounToken,
    major_cond_type: Option<ConditionalToken>,
    major_cond_sign: Option<bool>,
    major_cond_targets: Vec<Target>,
    minor_cond_type: Option<ConditionalToken>,
    minor_cond_sign: Option<bool>,
    minor_cond_target: Option<Target>,
    action_type: VerbToken,
    action_targets: Option<Vec<Target>>,
    action_target: Option<Target>,
    action_signs: Option<Vec<bool>>,
    action_sign: Option<bool>
}

// Adds a statement to the stream
pub fn append_statement(
    out: &mut Vec<Statement>, 
    subject: &NounToken, 
    major_cond_type: &Option<ConditionalToken>,
    major_cond_sign: &Option<bool>,
    major_cond_targets: Option<&[Target]>,
    minor_cond_type: &Option<ConditionalToken>,
    minor_cond_sign: &Option<bool>,
    minor_cond_target: &Option<Target>,
    action_type: &VerbToken,
    action_targets: &[Target],
    action_signs: &[bool],
    ) {
    // [NOUN] IS [NOUN] AND [NOUN] evaluates the AND statement *before* the IS, 
    // which means we can't guarantee that each target is its separate instruction.
    // [NOUN] IS [NOUN] AND [PROPERTY] evaluates as two separate instructions.
    if let VerbToken::Is = action_type {
        let mut start_index = 0;
        let total = action_targets.len();
        for (i, target) in action_targets.iter().enumerate() {
            if let Target::Noun(_) = target {
                //
            }
            else {
                // TODO split into 'targets' and 'target'
                let statement = Statement {
                    subject: *subject,
                    major_cond_type: *major_cond_type,
                    major_cond_sign: *major_cond_sign,
                    major_cond_targets: match major_cond_targets {
                        Some(v) => v.to_vec(),
                        None => Vec::new()
                    },
                    minor_cond_type: *minor_cond_type,
                    minor_cond_sign: *minor_cond_sign,
                    minor_cond_target: *minor_cond_target,
                    action_type: *action_type,
                    action_targets: Some(action_targets[start_index..i].to_vec()),
                    action_target: None,
                    action_signs: Some(action_signs.to_vec()),
                    action_sign: None,
                };
                out.push(statement);
                start_index = i;
            }
        }
        if start_index != total {
            // TODO split into 'targets' and 'target'
            let statement = Statement {
                subject: *subject,
                major_cond_type: *major_cond_type,
                major_cond_sign: *major_cond_sign,
                major_cond_targets: match major_cond_targets {
                    Some(v) => v.to_vec(),
                    None => Vec::new()
                },
                minor_cond_type: *minor_cond_type,
                minor_cond_sign: *minor_cond_sign,
                minor_cond_target: *minor_cond_target,
                action_type: *action_type,
                action_targets: Some(action_targets[start_index..].to_vec()),
                action_target: None,
                action_signs: Some(action_signs.to_vec()),
                action_sign: None
            };
            out.push(statement);
        }
    }
    else {
        // For verbs other than IS, each AND X is guaranteed
        // to be a separate instruction.
        // TODO split into 'targets' and 'target'
        for (i, target) in action_targets.iter().enumerate() {
            let statement = Statement {
                subject: *subject,
                major_cond_type: *major_cond_type,
                major_cond_sign: *major_cond_sign,
                major_cond_targets: match major_cond_targets {
                    Some(v) => v.to_vec(),
                    None => Vec::new()
                },
                minor_cond_type: *minor_cond_type,
                minor_cond_sign: *minor_cond_sign,
                minor_cond_target: *minor_cond_target,
                action_type: *action_type,
                action_targets: None,
                action_target: Some(*target),
                action_signs: None,
                action_sign: Some(action_signs[i])
            };
            out.push(statement);
        }
    }
}

/// Parses a stream of Baba tokens into a stream of statements.
/// 
pub fn parse(tokens: &[LexToken]) -> Vec<Statement> {
    let mut out = Vec::new();
    let mut state = ParserState::Blank;

    // Used to construct statements bit-by-bit
    let mut subject: Option<NounToken> = None;
    let mut major_cond: Option<ConditionalToken> = None;
    let mut major_cond_sign = false;
    let mut major_cond_targets: Vec<Target> = Vec::new();
    let mut minor_cond: Option<ConditionalToken> = None;
    let mut minor_cond_sign = false;
    let mut minor_cond_target: Option<Target> = None;
    let mut action_type: Option<VerbToken> = None;
    let mut action_targets: Vec<Target> = Vec::new();
    let mut action_sign = false;
    let mut action_signs: Vec<bool> = Vec::new();

    for token in tokens {
        // The compiler is hopefully smart enough to recognize
        // that this is a finite state machine
        match state {
            ParserState::Blank => {
                // Expect statements to begin with a noun
                if let LexToken::Noun(noun) = token {
                    subject = Some(*noun);
                    state = ParserState::Subject;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Noun, got {:?}", token)
                    );
                }
            },
            ParserState::Subject => {
                if let LexToken::Verb(verb) = token {
                    if let VerbToken::Is = verb {
                        state = ParserState::MajIs;
                    }
                    else {
                        state = ParserState::MajAct;
                    }
                    action_type = Some(*verb);
                    
                }
                else if let LexToken::Conditional(cond) = token {
                    // Facing
                    if let ConditionalToken::Facing = cond {
                        state = ParserState::MajCondFacing;
                    }
                    else {
                        state = ParserState::MajCond;
                    }
                    major_cond = Some(*cond);
                }
                else if let LexToken::Not = token {
                    major_cond_sign = !major_cond_sign;
                    state = ParserState::ExpectsMajCond;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Verb, Conditional or Not, got {:?}", token)
                    );
                }
            },
            ParserState::ExpectsMajCond => {
                if let LexToken::Conditional(cond) = token {
                    // FACING can be followed by a directional property as well as nouns
                    if let ConditionalToken::Facing = cond {
                        state = ParserState::MajCondFacing;
                    }
                    // Other conditionals are followed by nouns
                    else {
                        state = ParserState::MajCond;
                    }
                    major_cond = Some(*cond);
                }
                else if let LexToken::Not = token {
                    // NOT NOT cancels itself out
                    major_cond_sign = !major_cond_sign;
                    state = ParserState::ExpectsMajCond;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Conditional or Not, got {:?}", token)
                    );
                }
            },
            ParserState::MajCond => {
                if let LexToken::Noun(noun) = token {
                    // Nouns and properties are wrapped with an enum due to FACING
                    major_cond_targets.push(Target::Noun(*noun));
                    state = ParserState::MajCondTarget;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Noun, got {:?}", token)
                    );
                }
            },
            ParserState::MajCondFacing => {
                if let LexToken::Noun(noun) = token {
                    // Nouns and properties are wrapped with an enum due to FACING
                    major_cond_targets.push(Target::Noun(*noun));
                    state = ParserState::MajCondFacingTarget;
                }
                else if let LexToken::Property(prop) = token {
                    // FACING accepts UP, DOWN, LEFT, RIGHT
                    match prop {
                        PropertyToken::Up | PropertyToken::Down | PropertyToken::Left | PropertyToken::Right => {
                            major_cond_targets.push(Target::Property(*prop))
                        },
                        _ => {
                            throw_error(
                                ErrorType::StatementParserError, 
                                &format!(
                                    "Property words following Facing must be Up, Down, Left or Right, not {:?}",
                                    prop
                                )
                            )
                        }
                    }
                    state = ParserState::MajCondFacingTarget;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Noun or Property, got {:?}", token)
                    );
                }
            },
            ParserState::MajCondTarget => {
                if let LexToken::Verb(verb) = token {
                    if let VerbToken::Is = verb {
                        state = ParserState::MajIs;
                    }
                    else {
                        state = ParserState::MajAct;
                    }
                    action_type = Some(*verb);
                }
                else if let LexToken::And = token {
                    state = ParserState::CondAnd;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Verb or Not, got {:?}", token)
                    );
                }
            },
            ParserState::MajCondFacingTarget => {
                if let LexToken::Verb(verb) = token {
                    if let VerbToken::Is = verb {
                        state = ParserState::MajIs;
                    }
                    else {
                        state = ParserState::MajAct;
                    }
                    action_type = Some(*verb);
                }
                else if let LexToken::And = token {
                    state = ParserState::CondFacingAnd;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Verb or Not, got {:?}", token)
                    );
                }
            },
            ParserState::CondAnd => {
                if let LexToken::Noun(noun) = token {
                    major_cond_targets.push(Target::Noun(*noun));
                    state = ParserState::MajCondTarget;
                }
                else if let LexToken::Conditional(cond) = token {
                    // Minor conditionals also account for FACING
                    if let ConditionalToken::Facing = cond {
                        state = ParserState::MinCondFacing;
                    }
                    else {
                        state = ParserState::MinCond;
                    }
                    minor_cond = Some(*cond);
                }
                else if let LexToken::Not = token {
                    minor_cond_sign = !minor_cond_sign;
                    state = ParserState::ExpectsMinCond;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Verb or Not, got {:?}", token)
                    );
                }
            },
            ParserState::CondFacingAnd => {
                if let LexToken::Noun(noun) = token {
                    major_cond_targets.push(Target::Noun(*noun));
                    state = ParserState::MajCondTarget;
                }
                else if let LexToken::Property(prop) = token {
                    match prop {
                        PropertyToken::Up | PropertyToken::Down | PropertyToken::Left | PropertyToken::Right => {
                            major_cond_targets.push(Target::Property(*prop))
                        },
                        _ => {
                            throw_error(
                                ErrorType::StatementParserError, 
                                &format!(
                                    "Property words following Facing must be Up, Down, Left or Right, not {:?}",
                                    prop
                                )
                            )
                        }
                    }
                    state = ParserState::MajCondFacingTarget;
                }
                else if let LexToken::Conditional(cond) = token {
                    // Minor conditionals also account for FACING
                    if let ConditionalToken::Facing = cond {
                        state = ParserState::MinCondFacing;
                    }
                    else {
                        state = ParserState::MinCond;
                    }
                    minor_cond = Some(*cond);
                }
                else if let LexToken::Not = token {
                    minor_cond_sign = !minor_cond_sign;
                    state = ParserState::ExpectsMinCond;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Noun, Property, Conditional or Not, got {:?}", token)
                    );
                }
            },
            ParserState::ExpectsMinCond => {
                if let LexToken::Conditional(cond) = token {
                    if let ConditionalToken::Facing = cond {
                        state = ParserState::MinCondFacing;
                    }
                    else {
                        state = ParserState::MinCond;
                    }
                    minor_cond = Some(*cond);
                }
                else if let LexToken::Not = token {
                    minor_cond_sign = !minor_cond_sign;
                    state = ParserState::ExpectsMinCond;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Conditional or Not, got {:?}", token)
                    );
                }
            },
            ParserState::MinCond => {
                if let LexToken::Noun(noun) = token {
                    state = ParserState::MinCondTarget;
                    minor_cond_target = Some(Target::Noun(*noun));
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Noun, got {:?}", token)
                    );
                }
            },
            ParserState::MinCondFacing => {
                if let LexToken::Noun(noun) = token {
                    state = ParserState::MinCondTarget;
                    minor_cond_target = Some(Target::Noun(*noun));
                }
                else if let LexToken::Property(prop) = token {
                    match prop {
                        PropertyToken::Up | PropertyToken::Down | PropertyToken::Left | PropertyToken::Right => {
                            minor_cond_target = Some(Target::Property(*prop))
                        },
                        _ => {
                            throw_error(
                                ErrorType::StatementParserError, 
                                &format!(
                                    "Property words following Facing must be Up, Down, Left or Right, not {:?}",
                                    prop
                                )
                            )
                        }
                    }
                    state = ParserState::MinCondTarget;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Noun or Property, got {:?}", token)
                    );
                }
            },
            ParserState::MinCondTarget => {
                if let LexToken::Verb(verb) = token {
                    if let VerbToken::Is = verb {
                        state = ParserState::MajIs;
                    }
                    else {
                        state = ParserState::MajAct;
                    }
                    action_type = Some(*verb);
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Verb, got {:?}", token)
                    );
                }
            },
            ParserState::MajAct => {
                if let LexToken::Noun(noun) = token {
                    action_signs.push(action_sign);
                    action_targets.push(Target::Noun(*noun));
                    state = ParserState::MajActTarget;
                }
                else if let LexToken::Not = token {
                    action_sign = !action_sign;
                    state = ParserState::MajAct;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Noun or Not, got {:?}", token)
                    );
                }
            },
            ParserState::MajIs => {
                if let LexToken::Property(prop) = token {
                    action_signs.push(action_sign);
                    action_targets.push(Target::Property(*prop));
                    state = ParserState::MajIsTarget;
                }
                else if let LexToken::Noun(noun) = token {
                    action_signs.push(action_sign);
                    action_targets.push(Target::Noun(*noun));
                    state = ParserState::MajIsTarget;
                }
                else if let LexToken::Not = token {
                    action_sign = !action_sign;
                    state = ParserState::MajIs;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Property, Noun or Not, got {:?}", token)
                    );
                }
            },
            ParserState::MajActTarget => {
                // Starting a new statement
                if let LexToken::Noun(noun) = token {
                    append_statement(
                        &mut out,
                        &subject.clone().unwrap(), 
                        &major_cond, 
                        &Some(major_cond_sign), 
                        Some(&major_cond_targets),
                        &minor_cond, 
                        &Some(minor_cond_sign), 
                        &minor_cond_target, 
                        &action_type.unwrap(), 
                        &action_targets, 
                        &action_signs
                    );
                    action_targets.clear();
                    action_signs.clear();
                    major_cond_targets.clear();
                    major_cond_sign = false;
                    subject = Some(*noun);
                    state = ParserState::Subject;
                }
                else if let LexToken::And = token {
                    state = ParserState::ActAnd;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Noun or And, got {:?}", token)
                    );
                }
            },
            ParserState::MajIsTarget => {
                // Starting a new statement
                if let LexToken::Noun(noun) = token {
                    append_statement(
                        &mut out,
                        &subject.clone().unwrap(), 
                        &major_cond, 
                        &Some(major_cond_sign), 
                        Some(&major_cond_targets),
                        &minor_cond, 
                        &Some(minor_cond_sign), 
                        &minor_cond_target, 
                        &action_type.unwrap(), 
                        &action_targets, 
                        &action_signs
                    );
                    action_targets.clear();
                    action_signs.clear();
                    major_cond_targets.clear();
                    major_cond_sign = false;
                    subject = Some(*noun);
                    state = ParserState::Subject;
                }
                else if let LexToken::And = token {
                    state = ParserState::IsAnd;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Noun or And, got {:?}", token)
                    );
                }
            },
            ParserState::ActAnd => {
                // Prepending to an existing statement
                if let LexToken::Noun(noun) = token {
                    action_signs.push(action_sign);
                    action_targets.push(Target::Noun(*noun));
                    state = ParserState::MajActTarget;
                }
                else if let LexToken::Not = token {
                    action_sign = !action_sign;
                    state = ParserState::ExpectsMajActTarget;
                }
                else if let LexToken::Verb(verb) = token {
                    append_statement(
                        &mut out,
                        &subject.clone().unwrap(), 
                        &major_cond, 
                        &Some(major_cond_sign), 
                        Some(&major_cond_targets),
                        &minor_cond, 
                        &Some(minor_cond_sign), 
                        &minor_cond_target, 
                        &action_type.unwrap(), 
                        &action_targets, 
                        &action_signs
                    );
                    // Minor actions come after major actions.
                    // They occupy the same subject and conditionals
                    // as the original statement, so we only override
                    // the original action.
                    action_type = Some(*verb);
                    action_targets.clear();
                    action_signs.clear();
                    state = ParserState::ExpectsMinActTarget;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Noun, Not or Verb, got {:?}", token)
                    );
                }
            },
            ParserState::IsAnd => {
                // Prepending to an existing statement
                if let LexToken::Noun(noun) = token {
                    action_signs.push(action_sign);
                    action_targets.push(Target::Noun(*noun));
                    state = ParserState::MajIsTarget;
                }
                else if let LexToken::Property(prop) = token {
                    action_signs.push(action_sign);
                    action_targets.push(Target::Property(*prop));
                    state = ParserState::MajIsTarget;
                }
                else if let LexToken::Not = token {
                    action_sign = !action_sign;
                    state = ParserState::ExpectsMajIsTarget;
                }
                else if let LexToken::Verb(verb) = token {
                    append_statement(
                        &mut out,
                        &subject.clone().unwrap(), 
                        &major_cond, 
                        &Some(major_cond_sign), 
                        Some(&major_cond_targets),
                        &minor_cond, 
                        &Some(minor_cond_sign), 
                        &minor_cond_target, 
                        &action_type.unwrap(), 
                        &action_targets, 
                        &action_signs
                    );
                    // Minor actions come after major actions
                    action_type = Some(*verb);
                    action_targets.clear();
                    action_signs.clear();
                    state = ParserState::ExpectsMinActTarget;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Noun, Property, Not or Verb, got {:?}", token)
                    );
                }
            },
            ParserState::ExpectsMajActTarget => {
                // Prepending to an existing statement
                if let LexToken::Noun(noun) = token {
                    action_signs.push(action_sign);
                    action_targets.push(Target::Noun(*noun));
                    state = ParserState::MajActTarget;
                }
                else if let LexToken::Not = token {
                    action_sign = !action_sign;
                    state = ParserState::ExpectsMajActTarget;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Noun or Not, got {:?}", token)
                    );
                }
            },
            ParserState::ExpectsMajIsTarget => {
                // Prepending to an existing statement
                if let LexToken::Noun(noun) = token {
                    action_signs.push(action_sign);
                    action_targets.push(Target::Noun(*noun));
                    state = ParserState::MajIsTarget;
                }
                else if let LexToken::Property(prop) = token {
                    action_signs.push(action_sign);
                    action_targets.push(Target::Property(*prop));
                    state = ParserState::MajIsTarget;
                }
                else if let LexToken::Not = token {
                    action_sign = !action_sign;
                    state = ParserState::ExpectsMajIsTarget;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Noun, Propery or Not, got {:?}", token)
                    );
                }
            },
            // Minor actions can only have one target, and thus
            // it's not necessary to split this between IS and other verbs.
            ParserState::ExpectsMinActTarget => {
                // Prepending to an existing statement
                if let LexToken::Noun(noun) = token {
                    action_signs.push(action_sign);
                    action_targets.push(Target::Noun(*noun));
                    append_statement(
                        &mut out,
                        &subject.clone().unwrap(), 
                        &major_cond, 
                        &Some(major_cond_sign), 
                        Some(&major_cond_targets),
                        &minor_cond, 
                        &Some(minor_cond_sign), 
                        &minor_cond_target, 
                        &action_type.unwrap(), 
                        &action_targets, 
                        &action_signs
                    );
                    // It's not necessary to clear the subject
                    // or action type, as those are necessarily
                    // overriden by new statements.
                    action_signs.clear();
                    action_targets.clear();
                    major_cond_targets.clear();
                    major_cond_sign = false;
                    state = ParserState::Blank;
                }
                else if let LexToken::Property(prop) = token {
                    action_signs.push(action_sign);
                    action_targets.push(Target::Property(*prop));
                    append_statement(
                        &mut out,
                        &subject.clone().unwrap(), 
                        &major_cond, 
                        &Some(major_cond_sign), 
                        Some(&major_cond_targets),
                        &minor_cond, 
                        &Some(minor_cond_sign), 
                        &minor_cond_target, 
                        &action_type.unwrap(), 
                        &action_targets, 
                        &action_signs
                    );
                    action_signs.clear();
                    action_targets.clear();
                    major_cond_targets.clear();
                    major_cond_sign = false;
                    state = ParserState::Blank;
                }
                else if let LexToken::Not = token {
                    action_sign = !action_sign;
                    state = ParserState::ExpectsMinActTarget;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        &format!("Expected Noun, Propery or Not, got {:?}", token)
                    );
                }
            }
        }
    }
    // We've reached the end of our token stream, i.e. EOF.
    // If EOF came unexpectedly, we will error out.
    // Otherwise, we clean up after ourselves.
    match state {
        ParserState::Blank => {
            // No need to do anything
        },
        ParserState::MajActTarget => {
            // Finish the final statement
            append_statement(
                &mut out,
                &subject.clone().unwrap(), 
                &major_cond, 
                &Some(major_cond_sign), 
                Some(&major_cond_targets),
                &minor_cond, 
                &Some(minor_cond_sign), 
                &minor_cond_target, 
                &action_type.unwrap(), 
                &action_targets, 
                &action_signs
            );
        },
        ParserState::MajIsTarget => {
            // Finish the final statement
            append_statement(
                &mut out,
                &subject.clone().unwrap(), 
                &major_cond, 
                &Some(major_cond_sign), 
                Some(&major_cond_targets),
                &minor_cond, 
                &Some(minor_cond_sign), 
                &minor_cond_target, 
                &action_type.unwrap(), 
                &action_targets, 
                &action_signs
            );
        },
        _ => {
            // EOF during some other random state
            throw_error(
                ErrorType::StatementParserError,
                "Unexpected EOF during statement parsing"
            )
        }
    }
    
    out
}