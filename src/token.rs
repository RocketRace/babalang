use std::collections::HashMap;

// Valid tokens
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NounToken {
    All,
    Empty,
    Level,
    Image,
    Identifier(usize)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VerbToken {
    Eat,
    Fear,
    Follow,
    Has,
    Is,
    Make,
    Mimic,
    Play,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PropertyToken {
    You,
    Move,
    Text,
    Up,
    Down,
    Left,
    Right,
    Fall,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PrefixToken {
    Idle,
    Lonely
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConditionalToken {
    On,
    Near,
    Facing,
    Without
}

/// Every valid Baba token is a subset of LexToken.
#[derive(Debug, PartialEq, Clone)]
pub enum LexToken {
    Noun(NounToken),
    Verb(VerbToken),
    Property(PropertyToken),
    Prefix(PrefixToken),
    Not,
    And,
    Conditional(ConditionalToken)
}

/// Parses a char slice into the associated token. Returns None if the slice is empty.
/// If the token is a newly seen identifier, associates the identifier with an integer
/// in the HashMap provided.
/// 
/// # Arguments
/// 
/// * `buffer` - A character slice to parse the token from.
/// 
/// * `identifiers` - A HashMap that associates each unique token identifer to an usize.
pub fn parse<'a>(buffer: &'a [u8], identifiers: &mut HashMap<String, usize>) -> Option<LexToken> {
    if buffer.len() == 0 {
        None
    }
    else {
        let raw = std::str::from_utf8(buffer).unwrap();
        let id: &str = &raw.to_ascii_lowercase(); // Language is case independent
        let token = match id {
            // NounToken keywords
            "all" => LexToken::Noun(NounToken::All),
            "empty" => LexToken::Noun(NounToken::Empty),
            "level" => LexToken::Noun(NounToken::Level),
            "image" => LexToken::Noun(NounToken::Image),
            // Verb keywords
            "eat" => LexToken::Verb(VerbToken::Eat),
            "fear" => LexToken::Verb(VerbToken::Fear),
            "follow" => LexToken::Verb(VerbToken::Follow),
            "has" => LexToken::Verb(VerbToken::Has),
            "is" => LexToken::Verb(VerbToken::Is),
            "make" => LexToken::Verb(VerbToken::Make),
            "mimic" => LexToken::Verb(VerbToken::Mimic),
            "play" => LexToken::Verb(VerbToken::Play),
            // Property keywords
            "down" => LexToken::Property(PropertyToken::Down),
            "fall" => LexToken::Property(PropertyToken::Fall),
            "left" => LexToken::Property(PropertyToken::Left),
            "move" => LexToken::Property(PropertyToken::Move),
            "right" => LexToken::Property(PropertyToken::Right),
            "text" => LexToken::Property(PropertyToken::Text),
            "up" => LexToken::Property(PropertyToken::Up),
            "you" => LexToken::Property(PropertyToken::You),
            // Prefix keywords 
            "idle" => LexToken::Prefix(PrefixToken::Idle),
            "lonely" => LexToken::Prefix(PrefixToken::Lonely),
            // "And"
            "and" => LexToken::And,
            // "Not"
            "not" => LexToken::Not,
            // "Conditional" keywords
            "facing" => LexToken::Conditional(ConditionalToken::Facing),
            "near" => LexToken::Conditional(ConditionalToken::Near),
            "on" => LexToken::Conditional(ConditionalToken::On),
            "without" => LexToken::Conditional(ConditionalToken::Without),
            // Everything else (identifiers)
            _ => {
                // Lazily hashes a string, returning an identifier (usize)
                let hash = match identifiers.get(id) {
                    // For existing strings
                    Some(n) => *n,
                    // For new strings, the unique identifier is just the length 
                    // of the set, i.e. each identifier is one greater than the previous.
                    None => {
                        let new_id = identifiers.len();
                        identifiers.insert(String::from(id), new_id);
                        new_id
                    },
                };

                LexToken::Noun(NounToken::Identifier(hash))
            }
        };
        Some(token)
    }
}

/// Token parsing tests
#[cfg(test)]
mod tests {
    use crate::token::{parse, LexToken, NounToken, VerbToken, PropertyToken, PrefixToken, ConditionalToken};
    use std::collections::HashMap;
    #[test]
    fn parse_keywords_all() {
        // Line breaks are not significant here, since this test filters them out
        let string = "all empty eat fear follow has is make mimic play 
        down left move right text up you idle lonely and not facing near on without";
        
        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<LexToken> = words.iter().map(|&x| 
            parse(x.as_bytes(), &mut identifiers).unwrap()).collect();
        
        assert_eq!(
            tokens,
            vec![
                LexToken::Noun(NounToken::All),
                LexToken::Noun(NounToken::Empty),
                LexToken::Verb(VerbToken::Eat),
                LexToken::Verb(VerbToken::Fear),
                LexToken::Verb(VerbToken::Follow),
                LexToken::Verb(VerbToken::Has),
                LexToken::Verb(VerbToken::Is),
                LexToken::Verb(VerbToken::Make),
                LexToken::Verb(VerbToken::Mimic),
                LexToken::Verb(VerbToken::Play),
                LexToken::Property(PropertyToken::Down),
                LexToken::Property(PropertyToken::Left),
                LexToken::Property(PropertyToken::Move),
                LexToken::Property(PropertyToken::Right),
                LexToken::Property(PropertyToken::Text),
                LexToken::Property(PropertyToken::Up),
                LexToken::Property(PropertyToken::You),
                LexToken::Prefix(PrefixToken::Idle),
                LexToken::Prefix(PrefixToken::Lonely),
                LexToken::And,
                LexToken::Not,
                LexToken::Conditional(ConditionalToken::Facing),
                LexToken::Conditional(ConditionalToken::Near),
                LexToken::Conditional(ConditionalToken::On),
                LexToken::Conditional(ConditionalToken::Without),
            ]
        )
    }
    #[test]
    fn parse_keywords_duplicate() {
        let string = "is is is is is is is";
        
        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<LexToken> = words.iter().map(|&x| parse(x.as_bytes(), &mut identifiers).unwrap()).collect();

        assert_eq!(
            tokens,
            vec![
                LexToken::Verb(VerbToken::Is),
                LexToken::Verb(VerbToken::Is),
                LexToken::Verb(VerbToken::Is),
                LexToken::Verb(VerbToken::Is),
                LexToken::Verb(VerbToken::Is),
                LexToken::Verb(VerbToken::Is),
                LexToken::Verb(VerbToken::Is)
            ]
        )
    }
    #[test]
    fn parse_keywords_only() {
        // Line breaks are not significant here, since this test filters them out
        let string = "all empty eat fear follow has is make mimic play 
        down left move right text up you idle lonely and not facing near on without";
        
        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let _tokens: Vec<LexToken> = words.iter().map(|&x| parse(x.as_bytes(), &mut identifiers).unwrap()).collect();
        
        // None of the keywords should be parsed as identifiers
        assert_eq!(identifiers.len(), 0)
    }
    #[test]
    fn parse_keywords_mixed() {
        let string = "all empty is eat empty and and not is text up you and not all";
        
        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<LexToken> = words.iter().map(|&x| parse(x.as_bytes(), &mut identifiers).unwrap()).collect();
        
        assert_eq!(
            tokens,
            vec![
                LexToken::Noun(NounToken::All),
                LexToken::Noun(NounToken::Empty),
                LexToken::Verb(VerbToken::Is),
                LexToken::Verb(VerbToken::Eat),
                LexToken::Noun(NounToken::Empty),
                LexToken::And,
                LexToken::And,
                LexToken::Not,
                LexToken::Verb(VerbToken::Is),
                LexToken::Property(PropertyToken::Text),
                LexToken::Property(PropertyToken::Up),
                LexToken::Property(PropertyToken::You),
                LexToken::And,
                LexToken::Not,
                LexToken::Noun(NounToken::All)
            ]
        )
    }
    #[test]
    fn parse_identifiers_all() {
        let string = "baba keke me 42f test_identifier 0 ___ id";

        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<LexToken> = words.iter().map(|&x| parse(x.as_bytes(), &mut identifiers).unwrap()).collect();

        assert_eq!(
            tokens,
            vec![
                LexToken::Noun(NounToken::Identifier(0)),
                LexToken::Noun(NounToken::Identifier(1)),
                LexToken::Noun(NounToken::Identifier(2)),
                LexToken::Noun(NounToken::Identifier(3)),
                LexToken::Noun(NounToken::Identifier(4)),
                LexToken::Noun(NounToken::Identifier(5)),
                LexToken::Noun(NounToken::Identifier(6)),
                LexToken::Noun(NounToken::Identifier(7))
            ]
        )
    }
    #[test]
    fn parse_identifiers_duplicate() {
        let string = "baba baba baba baba baba baba baba baba";

        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<LexToken> = words.iter().map(|&x| parse(x.as_bytes(), &mut identifiers).unwrap()).collect();

        assert_eq!(
            tokens,
            vec![
                LexToken::Noun(NounToken::Identifier(0)),
                LexToken::Noun(NounToken::Identifier(0)),
                LexToken::Noun(NounToken::Identifier(0)),
                LexToken::Noun(NounToken::Identifier(0)),
                LexToken::Noun(NounToken::Identifier(0)),
                LexToken::Noun(NounToken::Identifier(0)),
                LexToken::Noun(NounToken::Identifier(0)),
                LexToken::Noun(NounToken::Identifier(0))
            ]
        )
    }
    #[test]
    fn parse_identifiers_mixed() {
        let string = "baba keke baba ___ me ___ keke baba";

        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<LexToken> = words.iter().map(|&x| parse(x.as_bytes(), &mut identifiers).unwrap()).collect();

        assert_eq!(
            tokens,
            vec![
                LexToken::Noun(NounToken::Identifier(0)),
                LexToken::Noun(NounToken::Identifier(1)),
                LexToken::Noun(NounToken::Identifier(0)),
                LexToken::Noun(NounToken::Identifier(2)),
                LexToken::Noun(NounToken::Identifier(3)),
                LexToken::Noun(NounToken::Identifier(2)),
                LexToken::Noun(NounToken::Identifier(1)),
                LexToken::Noun(NounToken::Identifier(0))
            ]
        )
    }

    #[test]
    fn parse_mixed() {
        let string = "baba and keke not on _ baba and 4 me is keke baba empty aaa";
        
        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<LexToken> = words.iter().map(|&x| parse(x.as_bytes(), &mut identifiers).unwrap()).collect();

        assert_eq!(tokens, 
            vec![
                LexToken::Noun(NounToken::Identifier(0)),
                LexToken::And,
                LexToken::Noun(NounToken::Identifier(1)),
                LexToken::Not,
                LexToken::Conditional(ConditionalToken::On),
                LexToken::Noun(NounToken::Identifier(2)),
                LexToken::Noun(NounToken::Identifier(0)),
                LexToken::And,
                LexToken::Noun(NounToken::Identifier(3)),
                LexToken::Noun(NounToken::Identifier(4)),
                LexToken::Verb(VerbToken::Is),
                LexToken::Noun(NounToken::Identifier(1)),
                LexToken::Noun(NounToken::Identifier(0)),
                LexToken::Noun(NounToken::Empty),
                LexToken::Noun(NounToken::Identifier(5))
            ]
        )
    }
}