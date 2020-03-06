use crate::token::{NounToken, PropertyToken, PrefixToken, VerbToken, ConditionalToken};

#[derive(Clone, Copy, Debug)]
pub enum Target {
    Noun(NounToken),
    Property(PropertyToken)
}

#[derive(Clone, Debug)]
pub struct Statement {
    pub prefix: Option<PrefixToken>,
    pub prefix_sign: Option<bool>,
    pub subject: NounToken,
    pub cond_type: Option<ConditionalToken>,
    pub cond_sign: Option<bool>,
    pub cond_targets: Vec<Target>,
    pub action_type: VerbToken,
    pub action_targets: Option<Vec<Target>>,
    pub action_target: Option<Target>,
    pub action_signs: Option<Vec<bool>>,
    pub action_sign: bool
}

// Adds a statement to the stream
pub fn append_statement(
    out: &mut Vec<Statement>, 
    prefix: &Option<PrefixToken>,
    prefix_sign: &Option<bool>,
    subject: &NounToken, 
    cond_type: &Option<ConditionalToken>,
    cond_sign: &Option<bool>,
    cond_targets: Option<&[Target]>,
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
                let one = i - start_index == 0;
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
                    action_targets: if one {None} else {Some(action_targets[start_index..i].to_vec())},
                    action_target: if one {Some(action_targets[start_index])} else {None},
                    action_signs: if one {None} else {Some(action_signs.to_vec())},
                    action_sign: if one {action_signs[0]} else {false},
                };
                out.push(statement);
                start_index = i;
            }
        }
        if start_index != total {
            let one = total - start_index == 0;
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
                action_targets: if one {None} else {Some(action_targets[start_index..].to_vec())},
                action_target: if one {Some(action_targets[start_index])} else {None},
                action_signs: if one {None} else {Some(action_signs.to_vec())},
                action_sign: if one {action_signs[0]} else {false},
            };
            out.push(statement);
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