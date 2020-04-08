use crate::token::{Noun, Property, Prefix, Verb, Conditional};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Target {
    Noun(Noun),
    Property(Property)
}

#[derive(Clone, Debug)]
pub struct Statement {
    pub prefix: Option<Prefix>,
    pub prefix_sign: Option<bool>,
    pub subject: Noun,
    pub cond_type: Option<Conditional>,
    pub cond_sign: Option<bool>,
    pub cond_targets: Vec<Target>,
    pub action_type: Verb,
    // These can only be nouns. 
    // Any properties will get converted into separate statements with `action_target`.
    pub action_targets: Option<Vec<Noun>>,
    pub action_target: Option<Target>,
    pub action_signs: Option<Vec<bool>>,
    pub action_sign: bool
}

// Adds a statement to the stream
pub fn append_statement(
    out: &mut Vec<Statement>, 
    prefix: &Option<Prefix>,
    prefix_sign: &Option<bool>,
    subject: &Noun, 
    cond_type: &Option<Conditional>,
    cond_sign: &Option<bool>,
    cond_targets: Option<&[Target]>,
    action_type: &Verb,
    action_targets: &[Target],
    action_signs: &[bool],
    ) {
    // [NOUN] IS [NOUN] AND [NOUN] evaluates the AND statement *before* the IS, 
    // which means we can't guarantee that each target is its separate instruction.
    // [NOUN] IS [NOUN] AND [PROPERTY] evaluates as two separate instructions.
    // TODO just scrap the whole darn thing
    if let Verb::Is = action_type {
        let mut start_index = 0;
        let total = action_targets.len();
        for (i, target) in action_targets.iter().enumerate() {
            match target {
                Target::Noun(n) if matches!(n, Noun::Identifier(_)) | matches!(n, Noun::All) => (),
                _ => {
                    match i - start_index {
                        0 => {
                            // Previously there was either nothing or a property
                            out.push(Statement {
                                prefix: *prefix,
                                prefix_sign: *prefix_sign,
                                subject: *subject,
                                cond_type: *cond_type,
                                cond_sign: *cond_sign,
                                cond_targets: match cond_targets {
                                    Some(v) => v.to_vec(),
                                    None => Vec::new()
                                },
                                action_type: *action_type,
                                action_targets: None,
                                action_target: Some(*target),
                                action_signs: None,
                                action_sign: action_signs[i],
                            });
                        },
                        1 => {
                            // Previously ignored single noun in AND chain
                            out.push(Statement {
                                prefix: *prefix,
                                prefix_sign: *prefix_sign,
                                subject: *subject,
                                cond_type: *cond_type,
                                cond_sign: *cond_sign,
                                cond_targets: match cond_targets {
                                    Some(v) => v.to_vec(),
                                    None => Vec::new()
                                },
                                action_type: *action_type,
                                action_targets: None,
                                action_target: Some(action_targets[i - 1]),
                                action_signs: None,
                                action_sign: action_signs[i - 1],
                            });
                            // Current property
                            out.push(Statement {
                                prefix: *prefix,
                                prefix_sign: *prefix_sign,
                                subject: *subject,
                                cond_type: *cond_type,
                                cond_sign: *cond_sign,
                                cond_targets: match cond_targets {
                                    Some(v) => v.to_vec(),
                                    None => Vec::new()
                                },
                                action_type: *action_type,
                                action_targets: None,
                                action_target: Some(*target),
                                action_signs: None,
                                action_sign: action_signs[i],
                            });
                        },
                        k if k > 1 => {
                            // Collect all nouns, discard properties 
                            // (there should never be properties here in the first place)
                            let mut targets = Vec::new();
                            for target in action_targets[i - k..i].iter() {
                                if let Target::Noun(noun) = target {
                                    targets.push(*noun);
                                }
                            }
                            // Previously ignored *multiple* nouns in AND chain
                            out.push(Statement {
                                prefix: *prefix,
                                prefix_sign: *prefix_sign,
                                subject: *subject,
                                cond_type: *cond_type,
                                cond_sign: *cond_sign,
                                cond_targets: match cond_targets {
                                    Some(v) => v.to_vec(),
                                    None => Vec::new()
                                },
                                action_type: *action_type,
                                action_targets: Some(targets),
                                action_target: None,
                                action_signs: Some(action_signs[i - k..i].to_vec()),
                                action_sign: false,
                            });
                            // Current property
                            out.push(Statement {
                                prefix: *prefix,
                                prefix_sign: *prefix_sign,
                                subject: *subject,
                                cond_type: *cond_type,
                                cond_sign: *cond_sign,
                                cond_targets: match cond_targets {
                                    Some(v) => v.to_vec(),
                                    None => Vec::new()
                                },
                                action_type: *action_type,
                                action_targets: None,
                                action_target: Some(*target),
                                action_signs: None,
                                action_sign: action_signs[i],
                            });
                        }
                        _ => ()
                    }
                    start_index = i + 1;
                }
            }
        }
        match total - start_index {
            1 => {
                out.push(Statement {
                    prefix: *prefix,
                    prefix_sign: *prefix_sign,
                    subject: *subject,
                    cond_type: *cond_type,
                    cond_sign: *cond_sign,
                    cond_targets: match cond_targets {
                        Some(v) => v.to_vec(),
                        None => Vec::new()
                    },
                    action_type: *action_type,
                    action_targets: None,
                    action_target: Some(action_targets[start_index]),
                    action_signs: None,
                    action_sign: action_signs[start_index],
                });
            },
            k if k > 1 => {
                let mut targets = Vec::new();
                for target in action_targets[start_index..].iter() {
                    if let Target::Noun(noun) = target {
                        targets.push(*noun);
                    }
                }
                out.push(Statement {
                    prefix: *prefix,
                    prefix_sign: *prefix_sign,
                    subject: *subject,
                    cond_type: *cond_type,
                    cond_sign: *cond_sign,
                    cond_targets: match cond_targets {
                        Some(v) => v.to_vec(),
                        None => Vec::new()
                    },
                    action_type: *action_type,
                    action_targets: Some(targets),
                    action_target: None,
                    action_signs: Some(action_signs[start_index..].to_vec()),
                    action_sign: false,
                });
            },
            _ => ()
        }
    }
    else {
        // For verbs other than IS, each AND X is guaranteed
        // to be a separate instruction.
        for (i, target) in action_targets.iter().enumerate() {
            let statement = Statement {
                prefix: *prefix,
                prefix_sign: *prefix_sign,
                subject: *subject,
                cond_type: *cond_type,
                cond_sign: *cond_sign,
                cond_targets: match cond_targets {
                    Some(v) => v.to_vec(),
                    None => Vec::new()
                },
                action_type: *action_type,
                action_targets: None,
                action_target: Some(*target),
                action_signs: None,
                action_sign: action_signs[i]
            };
            out.push(statement);
        }
    }
}