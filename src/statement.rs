use crate::token::{NounToken, PropertyToken, VerbToken, ConditionalToken};

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