use crate::token::{NounToken, VerbToken, PropertyToken, PrefixToken, ConditionalToken, LexToken};
use crate::statement::{Target, Statement, append_statement};
use crate::error_handler::{ErrorType, throw_error, throw_error_str};

/// The internal state of the statement parser.
#[derive(Debug)]
enum ParserState {
    Blank,
    // Subject & Prefix
    ExpectsPrefix, Prefix,
    Subject,
    // Major conditional 
    ExpectsMajCond, MajCond, MajCondTarget, CondAnd, 
    MajCondFacing, MajCondFacingTarget, CondFacingAnd,
    // Major action: IS
    MajAct, MajActTarget, ActAnd, ExpectsMajActTarget,
    // Major action: other verbs
    MajIs, MajIsTarget, IsAnd, ExpectsMajIsTarget,
    // Minor actions
    ExpectsMinActTarget,
}

/// Parses a stream of Baba tokens into a stream of statements.
/// Statements are parsed using a subset of the grammar used
/// in the original Baba Is You Game.
/// 
/// # Arguments
/// 
/// * `tokens` - A slice of tokens to read.
/// 
/// # Return
/// 
/// Returns a `Vec` of `Statement` objects.
pub fn parse(tokens: &[LexToken]) -> Vec<Statement> {
    let mut out = Vec::new();
    let mut state = ParserState::Blank;

    // Used to construct statements part-by-part
    let mut prefix: Option<PrefixToken> = None;
    let mut prefix_sign = false;
    let mut subject: Option<NounToken> = None;
    let mut cond_type: Option<ConditionalToken> = None;
    let mut cond_sign = false;
    let mut cond_targets: Vec<Target> = Vec::new();
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
                else if let LexToken::Prefix(pref) = token {
                    prefix = Some(*pref);
                    state = ParserState::Prefix;
                }
                else if let LexToken::Not = token {
                    prefix_sign = !prefix_sign;
                    state = ParserState::ExpectsPrefix;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        format!("Expected Noun, Prefix or Not, got {:?}", token)
                    );
                }
            },
            ParserState::ExpectsPrefix => {
                if let LexToken::Prefix(pref) = token {
                    prefix = Some(*pref);
                    state = ParserState::Prefix;
                }
                else if let LexToken::Not = token {
                    prefix_sign = !prefix_sign;
                    state = ParserState::ExpectsPrefix;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        format!("Expected Prefix or Not, got {:?}", token)
                    );
                }
            },
            ParserState::Prefix => {
                if let LexToken::Noun(noun) = token {
                    subject = Some(*noun);
                    state = ParserState::Subject;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        format!("Expected Noun, got {:?}", token)
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
                    cond_type = Some(*cond);
                }
                else if let LexToken::Not = token {
                    cond_sign = !cond_sign;
                    state = ParserState::ExpectsMajCond;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        format!("Expected Verb, Conditional or Not, got {:?}", token)
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
                    cond_type = Some(*cond);
                }
                else if let LexToken::Not = token {
                    // NOT NOT cancels itself out
                    cond_sign = !cond_sign;
                    state = ParserState::ExpectsMajCond;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        format!("Expected Conditional or Not, got {:?}", token)
                    );
                }
            },
            ParserState::MajCond => {
                if let LexToken::Noun(noun) = token {
                    // Nouns and properties are wrapped with an enum due to FACING
                    cond_targets.push(Target::Noun(*noun));
                    state = ParserState::MajCondTarget;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        format!("Expected Noun, got {:?}", token)
                    );
                }
            },
            ParserState::MajCondFacing => {
                if let LexToken::Noun(noun) = token {
                    // Nouns and properties are wrapped with an enum due to FACING
                    cond_targets.push(Target::Noun(*noun));
                    state = ParserState::MajCondFacingTarget;
                }
                else if let LexToken::Property(prop) = token {
                    // FACING accepts UP, DOWN, LEFT, RIGHT
                    match prop {
                        PropertyToken::Up | PropertyToken::Down | PropertyToken::Left | PropertyToken::Right => {
                            cond_targets.push(Target::Property(*prop))
                        },
                        _ => {
                            throw_error(
                                ErrorType::StatementParserError, 
                                format!(
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
                        format!("Expected Noun or Property, got {:?}", token)
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
                        format!("Expected Verb or And, got {:?}", token)
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
                        format!("Expected Verb or And, got {:?}", token)
                    );
                }
            },
            ParserState::CondAnd => {
                if let LexToken::Noun(noun) = token {
                    cond_targets.push(Target::Noun(*noun));
                    state = ParserState::MajCondTarget;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        format!("Expected Noun, got {:?}", token)
                    );
                }
            },
            ParserState::CondFacingAnd => {
                if let LexToken::Noun(noun) = token {
                    cond_targets.push(Target::Noun(*noun));
                    state = ParserState::MajCondTarget;
                }
                else if let LexToken::Property(prop) = token {
                    match prop {
                        PropertyToken::Up | PropertyToken::Down | PropertyToken::Left | PropertyToken::Right => {
                            cond_targets.push(Target::Property(*prop))
                        },
                        _ => {
                            throw_error(
                                ErrorType::StatementParserError, 
                                format!(
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
                        format!("Expected Noun or Property got {:?}", token)
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
                        format!("Expected Noun or Not, got {:?}", token)
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
                        format!("Expected Property, Noun or Not, got {:?}", token)
                    );
                }
            },
            ParserState::MajActTarget => {
                // Starting a new statement
                if let LexToken::Noun(noun) = token {
                    append_statement(
                        &mut out,
                        &prefix,
                        &Some(prefix_sign),
                        &subject.clone().unwrap(), 
                        &cond_type, 
                        &Some(cond_sign), 
                        Some(&cond_targets),
                        &action_type.unwrap(), 
                        &action_targets, 
                        &action_signs
                    );
                    action_targets.clear();
                    action_signs.clear();
                    cond_targets.clear();
                    cond_sign = false;
                    subject = Some(*noun);
                    state = ParserState::Subject;
                }
                // Continue existing statement (not IS)
                else if let LexToken::And = token {
                    state = ParserState::ActAnd;
                }
                // New statement (PREFIX)
                else if let LexToken::Prefix(pref) = token {
                    append_statement(
                        &mut out,
                        &prefix,
                        &Some(prefix_sign),
                        &subject.clone().unwrap(), 
                        &cond_type, 
                        &Some(cond_sign), 
                        Some(&cond_targets),
                        &action_type.unwrap(), 
                        &action_targets, 
                        &action_signs
                    );
                    action_targets.clear();
                    action_signs.clear();
                    cond_targets.clear();
                    cond_sign = false;
                    prefix = Some(*pref);
                    state = ParserState::Prefix;
                }
                // New statement (NOT PREFIX)
                else if let LexToken::Not = token {
                    append_statement(
                        &mut out,
                        &prefix,
                        &Some(prefix_sign),
                        &subject.clone().unwrap(), 
                        &cond_type, 
                        &Some(cond_sign), 
                        Some(&cond_targets),
                        &action_type.unwrap(), 
                        &action_targets, 
                        &action_signs
                    );
                    action_targets.clear();
                    action_signs.clear();
                    cond_targets.clear();
                    cond_sign = false;
                    prefix_sign = !prefix_sign;
                    state = ParserState::ExpectsPrefix;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        format!("Expected Noun, And, Prefix or Not, got {:?}", token)
                    );
                }
            },
            ParserState::MajIsTarget => {
                // Starting a new statement
                if let LexToken::Noun(noun) = token {
                    append_statement(
                        &mut out,
                        &prefix,
                        &Some(prefix_sign),
                        &subject.clone().unwrap(), 
                        &cond_type, 
                        &Some(cond_sign), 
                        Some(&cond_targets),
                        &action_type.unwrap(), 
                        &action_targets, 
                        &action_signs
                    );
                    action_targets.clear();
                    action_signs.clear();
                    cond_targets.clear();
                    cond_sign = false;
                    subject = Some(*noun);
                    state = ParserState::Subject;
                }
                // Continue existing statement (IS)
                else if let LexToken::And = token {
                    state = ParserState::IsAnd;
                }
                // New statement (PREFIX)
                else if let LexToken::Prefix(pref) = token {
                    append_statement(
                        &mut out,
                        &prefix,
                        &Some(prefix_sign),
                        &subject.clone().unwrap(), 
                        &cond_type, 
                        &Some(cond_sign), 
                        Some(&cond_targets),
                        &action_type.unwrap(), 
                        &action_targets, 
                        &action_signs
                    );
                    action_targets.clear();
                    action_signs.clear();
                    cond_targets.clear();
                    cond_sign = false;
                    prefix = Some(*pref);
                    state = ParserState::Prefix;
                }
                // New statement (NOT PREFIX)
                else if let LexToken::Not = token {
                    append_statement(
                        &mut out,
                        &prefix,
                        &Some(prefix_sign),
                        &subject.clone().unwrap(), 
                        &cond_type, 
                        &Some(cond_sign), 
                        Some(&cond_targets),
                        &action_type.unwrap(), 
                        &action_targets, 
                        &action_signs
                    );
                    action_targets.clear();
                    action_signs.clear();
                    cond_targets.clear();
                    cond_sign = false;
                    prefix_sign = !prefix_sign;
                    state = ParserState::ExpectsPrefix;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        format!("Expected Noun, And, Prefix, or Not, got {:?}", token)
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
                        &prefix,
                        &Some(prefix_sign),
                        &subject.clone().unwrap(), 
                        &cond_type, 
                        &Some(cond_sign), 
                        Some(&cond_targets),
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
                        format!("Expected Noun, Not or Verb, got {:?}", token)
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
                        &prefix,
                        &Some(prefix_sign),
                        &subject.clone().unwrap(), 
                        &cond_type, 
                        &Some(cond_sign), 
                        Some(&cond_targets),
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
                        format!("Expected Noun, Property, Not or Verb, got {:?}", token)
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
                        format!("Expected Noun or Not, got {:?}", token)
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
                        format!("Expected Noun, Propery or Not, got {:?}", token)
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
                        &prefix,
                        &Some(prefix_sign),
                        &subject.clone().unwrap(), 
                        &cond_type, 
                        &Some(cond_sign), 
                        Some(&cond_targets),
                        &action_type.unwrap(), 
                        &action_targets, 
                        &action_signs
                    );
                    // It's not necessary to clear the subject
                    // or action type, as those are necessarily
                    // overriden by new statements.
                    action_signs.clear();
                    action_targets.clear();
                    cond_targets.clear();
                    cond_sign = false;
                    state = ParserState::Blank;
                }
                else if let LexToken::Property(prop) = token {
                    action_signs.push(action_sign);
                    action_targets.push(Target::Property(*prop));
                    append_statement(
                        &mut out,
                        &prefix,
                        &Some(prefix_sign),
                        &subject.clone().unwrap(), 
                        &cond_type, 
                        &Some(cond_sign), 
                        Some(&cond_targets),
                        &action_type.unwrap(), 
                        &action_targets, 
                        &action_signs
                    );
                    action_signs.clear();
                    action_targets.clear();
                    cond_targets.clear();
                    cond_sign = false;
                    state = ParserState::Blank;
                }
                else if let LexToken::Not = token {
                    action_sign = !action_sign;
                    state = ParserState::ExpectsMinActTarget;
                }
                else {
                    throw_error(
                        ErrorType::StatementParserError,
                        format!("Expected Noun, Propery or Not, got {:?}", token)
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
                &prefix,
                &Some(prefix_sign),
                &subject.clone().unwrap(), 
                &cond_type, 
                &Some(cond_sign), 
                Some(&cond_targets),
                &action_type.unwrap(), 
                &action_targets, 
                &action_signs
            );
        },
        ParserState::MajIsTarget => {
            // Finish the final statement
            append_statement(
                &mut out,
                &prefix,
                &Some(prefix_sign),
                &subject.clone().unwrap(), 
                &cond_type, 
                &Some(cond_sign), 
                Some(&cond_targets),
                &action_type.unwrap(), 
                &action_targets, 
                &action_signs
            );
        },
        _ => {
            // EOF occurred during some other random state
            throw_error_str(
                ErrorType::StatementParserError,
                "Unexpected EOF during statement parsing"
            )
        }
    }
    
    out
}