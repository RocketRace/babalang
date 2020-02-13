use std::collections::HashMap;

// Valid tokens
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NounToken {
    All,
    Empty,
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
    Right
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConditionalToken {
    On,
    Near,
    Facing,
    Without
}

#[derive(Debug, PartialEq, Clone)]
pub enum LexToken {
    Noun(NounToken),
    Verb(VerbToken),
    Property(PropertyToken),
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
        let id = std::str::from_utf8(buffer).unwrap();
        let token = match id {
            // NounToken keywords
            "all" => LexToken::Noun(NounToken::All),
            "empty" => LexToken::Noun(NounToken::Empty),
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
            "left" => LexToken::Property(PropertyToken::Left),
            "move" => LexToken::Property(PropertyToken::Move),
            "right" => LexToken::Property(PropertyToken::Right),
            "text" => LexToken::Property(PropertyToken::Text),
            "up" => LexToken::Property(PropertyToken::Up),
            "you" => LexToken::Property(PropertyToken::You),
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

// ---TOKEN PARSING TESTS---
#[cfg(test)]
mod tests {
    use crate::token;
    use std::collections::HashMap;
    #[test]
    fn parse_keywords_all() {
        // Line breaks are not significant here, since this test filters them out
        let string = "all empty eat fear follow has is make mimic play 
        down left move right text up you and not facing near on without";
        
        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<token::LexToken> = words.iter().map(|&x| token::parse(x.as_bytes(), &mut identifiers).unwrap()).collect();
        
        assert_eq!(
            tokens,
            vec![
                token::LexToken::Noun(token::NounToken::All),
                token::LexToken::Noun(token::NounToken::Empty),
                token::LexToken::Verb(token::VerbToken::Eat),
                token::LexToken::Verb(token::VerbToken::Fear),
                token::LexToken::Verb(token::VerbToken::Follow),
                token::LexToken::Verb(token::VerbToken::Has),
                token::LexToken::Verb(token::VerbToken::Is),
                token::LexToken::Verb(token::VerbToken::Make),
                token::LexToken::Verb(token::VerbToken::Mimic),
                token::LexToken::Verb(token::VerbToken::Play),
                token::LexToken::Property(token::PropertyToken::Down),
                token::LexToken::Property(token::PropertyToken::Left),
                token::LexToken::Property(token::PropertyToken::Move),
                token::LexToken::Property(token::PropertyToken::Right),
                token::LexToken::Property(token::PropertyToken::Text),
                token::LexToken::Property(token::PropertyToken::Up),
                token::LexToken::Property(token::PropertyToken::You),
                token::LexToken::And,
                token::LexToken::Not,
                token::LexToken::Conditional(token::ConditionalToken::Facing),
                token::LexToken::Conditional(token::ConditionalToken::Near),
                token::LexToken::Conditional(token::ConditionalToken::On),
                token::LexToken::Conditional(token::ConditionalToken::Without),
            ]
        )
    }
    #[test]
    fn parse_keywords_duplicate() {
        let string = "is is is is is is is";
        
        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<token::LexToken> = words.iter().map(|&x| token::parse(x.as_bytes(), &mut identifiers).unwrap()).collect();

        assert_eq!(
            tokens,
            vec![
                token::LexToken::Verb(token::VerbToken::Is),
                token::LexToken::Verb(token::VerbToken::Is),
                token::LexToken::Verb(token::VerbToken::Is),
                token::LexToken::Verb(token::VerbToken::Is),
                token::LexToken::Verb(token::VerbToken::Is),
                token::LexToken::Verb(token::VerbToken::Is),
                token::LexToken::Verb(token::VerbToken::Is)
            ]
        )
    }
    #[test]
    fn parse_keywords_only() {
        // Line breaks are not significant here, since this test filters them out
        let string = "all empty eat fear follow has is make mimic play 
        down left move right text up you and not facing near on without";
        
        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let _tokens: Vec<token::LexToken> = words.iter().map(|&x| token::parse(x.as_bytes(), &mut identifiers).unwrap()).collect();
        
        // None of the keywords should be parsed as identifiers
        assert_eq!(identifiers.len(), 0)
    }
    #[test]
    fn parse_keywords_mixed() {
        let string = "all empty is eat empty and and not is text up you and not all";
        
        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<token::LexToken> = words.iter().map(|&x| token::parse(x.as_bytes(), &mut identifiers).unwrap()).collect();
        
        assert_eq!(
            tokens,
            vec![
                token::LexToken::Noun(token::NounToken::All),
                token::LexToken::Noun(token::NounToken::Empty),
                token::LexToken::Verb(token::VerbToken::Is),
                token::LexToken::Verb(token::VerbToken::Eat),
                token::LexToken::Noun(token::NounToken::Empty),
                token::LexToken::And,
                token::LexToken::And,
                token::LexToken::Not,
                token::LexToken::Verb(token::VerbToken::Is),
                token::LexToken::Property(token::PropertyToken::Text),
                token::LexToken::Property(token::PropertyToken::Up),
                token::LexToken::Property(token::PropertyToken::You),
                token::LexToken::And,
                token::LexToken::Not,
                token::LexToken::Noun(token::NounToken::All)
            ]
        )
    }
    #[test]
    fn parse_identifiers_all() {
        let string = "baba keke me 42f test_identifier 0 ___ id";

        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<token::LexToken> = words.iter().map(|&x| token::parse(x.as_bytes(), &mut identifiers).unwrap()).collect();

        assert_eq!(
            tokens,
            vec![
                token::LexToken::Noun(token::NounToken::Identifier(0)),
                token::LexToken::Noun(token::NounToken::Identifier(1)),
                token::LexToken::Noun(token::NounToken::Identifier(2)),
                token::LexToken::Noun(token::NounToken::Identifier(3)),
                token::LexToken::Noun(token::NounToken::Identifier(4)),
                token::LexToken::Noun(token::NounToken::Identifier(5)),
                token::LexToken::Noun(token::NounToken::Identifier(6)),
                token::LexToken::Noun(token::NounToken::Identifier(7))
            ]
        )
    }
    #[test]
    fn parse_identifiers_duplicate() {
        let string = "baba baba baba baba baba baba baba baba";

        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<token::LexToken> = words.iter().map(|&x| token::parse(x.as_bytes(), &mut identifiers).unwrap()).collect();

        assert_eq!(
            tokens,
            vec![
                token::LexToken::Noun(token::NounToken::Identifier(0)),
                token::LexToken::Noun(token::NounToken::Identifier(0)),
                token::LexToken::Noun(token::NounToken::Identifier(0)),
                token::LexToken::Noun(token::NounToken::Identifier(0)),
                token::LexToken::Noun(token::NounToken::Identifier(0)),
                token::LexToken::Noun(token::NounToken::Identifier(0)),
                token::LexToken::Noun(token::NounToken::Identifier(0)),
                token::LexToken::Noun(token::NounToken::Identifier(0))
            ]
        )
    }
    #[test]
    fn parse_identifiers_mixed() {
        let string = "baba keke baba ___ me ___ keke baba";

        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<token::LexToken> = words.iter().map(|&x| token::parse(x.as_bytes(), &mut identifiers).unwrap()).collect();

        assert_eq!(
            tokens,
            vec![
                token::LexToken::Noun(token::NounToken::Identifier(0)),
                token::LexToken::Noun(token::NounToken::Identifier(1)),
                token::LexToken::Noun(token::NounToken::Identifier(0)),
                token::LexToken::Noun(token::NounToken::Identifier(2)),
                token::LexToken::Noun(token::NounToken::Identifier(3)),
                token::LexToken::Noun(token::NounToken::Identifier(2)),
                token::LexToken::Noun(token::NounToken::Identifier(1)),
                token::LexToken::Noun(token::NounToken::Identifier(0))
            ]
        )
    }
}