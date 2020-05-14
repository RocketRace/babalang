use std::collections::HashMap;

// Valid tokens
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Noun {
    All,
    Empty,
    Level,
    Image,
    Identifier(usize)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Verb {
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
pub enum Property {
    // Primitives
    You,
    You2,
    Group,
    Tele,
    // Static
    Float,
    // Exit scope
    Done,
    // I/O
    Text,
    Word,
    // Program
    Win,
    Defeat,
    Sleep,
    // YOU / YOU2
    Move,
    Turn,
    Fall,
    More,
    Up,
    Down,
    Left,
    Right,
    // GROUP
    Shift,
    Sink,
    Swap,
    // LEVEL
    Power,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Prefix {
    Idle,
    Lonely
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Conditional {
    On,
    Near,
    Facing,
    Without
}

/// Every valid Baba token is a subset of Token.
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Noun(Noun),
    Verb(Verb),
    Property(Property),
    Prefix(Prefix),
    Not,
    And,
    Conditional(Conditional)
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
pub fn parse<'a>(buffer: &'a [u8], identifiers: &mut HashMap<usize, String>) -> Option<Token> {
    if buffer.len() == 0 {
        None
    }
    else {
        let raw = std::str::from_utf8(buffer).unwrap();
        let id: &str = &raw.to_ascii_lowercase(); // Language is case independent
        let token = match id {
            // Noun keywords
            "all" => Token::Noun(Noun::All),
            "empty" => Token::Noun(Noun::Empty),
            "level" => Token::Noun(Noun::Level),
            "image" => Token::Noun(Noun::Image),
            // Verb keywords
            "eat" => Token::Verb(Verb::Eat),
            "fear" => Token::Verb(Verb::Fear),
            "follow" => Token::Verb(Verb::Follow),
            "has" => Token::Verb(Verb::Has),
            "is" => Token::Verb(Verb::Is),
            "make" => Token::Verb(Verb::Make),
            "mimic" => Token::Verb(Verb::Mimic),
            "play" => Token::Verb(Verb::Play),
            // Property keywords
            // - Initializers
            "you" => Token::Property(Property::You),
            "you2" => Token::Property(Property::You2),
            "group" => Token::Property(Property::Group),
            "tele" => Token::Property(Property::Tele),
            // - Static
            "float" => Token::Property(Property::Float),
            // - I/O
            "text" => Token::Property(Property::Text),
            "word" => Token::Property(Property::Word),
            // - Program
            "win" => Token::Property(Property::Win),
            "defeat" => Token::Property(Property::Defeat),
            "sleep" => Token::Property(Property::Sleep),
            // - Other
            "done" => Token::Property(Property::Done),
            // - You
            "move" => Token::Property(Property::Move),
            "turn" => Token::Property(Property::Turn),
            "fall" => Token::Property(Property::Fall),
            "more" => Token::Property(Property::More),
            "right" => Token::Property(Property::Right),
            "up" => Token::Property(Property::Up),
            "left" => Token::Property(Property::Left),
            "down" => Token::Property(Property::Down),
            // - Group
            "shift" => Token::Property(Property::Shift),
            "sink" => Token::Property(Property::Sink),
            "swap" => Token::Property(Property::Swap),
            // - Level
            "power" => Token::Property(Property::Power),
            // Prefix keywords 
            "idle" => Token::Prefix(Prefix::Idle),
            "lonely" => Token::Prefix(Prefix::Lonely),
            // "And"
            "and" => Token::And,
            // "Not"
            "not" => Token::Not,
            // "Conditional" keywords
            "facing" => Token::Conditional(Conditional::Facing),
            "near" => Token::Conditional(Conditional::Near),
            "on" => Token::Conditional(Conditional::On),
            "without" => Token::Conditional(Conditional::Without),
            // Everything else (identifiers)
            _ => {
                let mut unique = true;
                let mut existing_id = 0;
                for (value, identifier) in identifiers.iter() {
                    if &id == &identifier {
                        existing_id = *value;
                        unique = false;
                        break;
                    }
                }
                if unique {
                    let new_id = identifiers.len();
                    // For new strings, the unique identifier is just the length 
                    // of the set, i.e. each identifier is one grer than the previous.
                    identifiers.insert(new_id, id.to_string());
                    Token::Noun(Noun::Identifier(new_id))
                }
                else {
                    Token::Noun(Noun::Identifier(existing_id))
                }
            }
        };
        Some(token)
    }
}

/// Token parsing tests
#[cfg(test)]
mod tests {
    use crate::token::{parse, Token, Noun, Verb, Property, Prefix, Conditional};
    use std::collections::HashMap;
    #[test]
    fn parse_keywords_all() {
        // Line breaks are not significant here, since this test filters them out
        let string = "all empty  fear follow has is make mimic play 
        down left move right text up you idle lonely and not facing near on without";
        
        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<Token> = words.iter().map(|&x| 
            parse(x.as_bytes(), &mut identifiers).unwrap()).collect();
        
        assert_eq!(
            tokens,
            vec![
                Token::Noun(Noun::All),
                Token::Noun(Noun::Empty),
                Token::Verb(Verb::Fear),
                Token::Verb(Verb::Follow),
                Token::Verb(Verb::Has),
                Token::Verb(Verb::Is),
                Token::Verb(Verb::Make),
                Token::Verb(Verb::Mimic),
                Token::Verb(Verb::Play),
                Token::Property(Property::Down),
                Token::Property(Property::Left),
                Token::Property(Property::Move),
                Token::Property(Property::Right),
                Token::Property(Property::Text),
                Token::Property(Property::Up),
                Token::Property(Property::You),
                Token::Prefix(Prefix::Idle),
                Token::Prefix(Prefix::Lonely),
                Token::And,
                Token::Not,
                Token::Conditional(Conditional::Facing),
                Token::Conditional(Conditional::Near),
                Token::Conditional(Conditional::On),
                Token::Conditional(Conditional::Without),
            ]
        )
    }
    #[test]
    fn parse_keywords_duplicate() {
        let string = "is is is is is is is";
        
        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<Token> = words.iter().map(|&x| parse(x.as_bytes(), &mut identifiers).unwrap()).collect();

        assert_eq!(
            tokens,
            vec![
                Token::Verb(Verb::Is),
                Token::Verb(Verb::Is),
                Token::Verb(Verb::Is),
                Token::Verb(Verb::Is),
                Token::Verb(Verb::Is),
                Token::Verb(Verb::Is),
                Token::Verb(Verb::Is)
            ]
        )
    }
    #[test]
    fn parse_keywords_only() {
        // Line breaks are not significant here, since this test filters them out
        let string = "all empty fear follow has is make mimic play 
        down left move right text up you idle lonely and not facing near on without";
        
        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let _s: Vec<Token> = words.iter().map(|&x| parse(x.as_bytes(), &mut identifiers).unwrap()).collect();
        
        // None of the keywords should be parsed as identifiers
        assert_eq!(identifiers.len(), 0)
    }
    #[test]
    fn parse_keywords_mixed() {
        let string = "all empty is  empty and and not is text up you and not all";
        
        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<Token> = words.iter().map(|&x| parse(x.as_bytes(), &mut identifiers).unwrap()).collect();
        
        assert_eq!(
            tokens,
            vec![
                Token::Noun(Noun::All),
                Token::Noun(Noun::Empty),
                Token::Verb(Verb::Is),
                Token::Noun(Noun::Empty),
                Token::And,
                Token::And,
                Token::Not,
                Token::Verb(Verb::Is),
                Token::Property(Property::Text),
                Token::Property(Property::Up),
                Token::Property(Property::You),
                Token::And,
                Token::Not,
                Token::Noun(Noun::All)
            ]
        )
    }
    #[test]
    fn parse_identifiers_all() {
        let string = "baba keke me 42f test_identifier 0 ___ id";

        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<Token> = words.iter().map(|&x| parse(x.as_bytes(), &mut identifiers).unwrap()).collect();

        assert_eq!(
            tokens,
            vec![
                Token::Noun(Noun::Identifier(0)),
                Token::Noun(Noun::Identifier(1)),
                Token::Noun(Noun::Identifier(2)),
                Token::Noun(Noun::Identifier(3)),
                Token::Noun(Noun::Identifier(4)),
                Token::Noun(Noun::Identifier(5)),
                Token::Noun(Noun::Identifier(6)),
                Token::Noun(Noun::Identifier(7))
            ]
        )
    }
    #[test]
    fn parse_identifiers_duplicate() {
        let string = "baba baba baba baba baba baba baba baba";

        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<Token> = words.iter().map(|&x| parse(x.as_bytes(), &mut identifiers).unwrap()).collect();

        assert_eq!(
            tokens,
            vec![
                Token::Noun(Noun::Identifier(0)),
                Token::Noun(Noun::Identifier(0)),
                Token::Noun(Noun::Identifier(0)),
                Token::Noun(Noun::Identifier(0)),
                Token::Noun(Noun::Identifier(0)),
                Token::Noun(Noun::Identifier(0)),
                Token::Noun(Noun::Identifier(0)),
                Token::Noun(Noun::Identifier(0))
            ]
        )
    }
    #[test]
    fn parse_identifiers_mixed() {
        let string = "baba keke baba ___ me ___ keke baba";

        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<Token> = words.iter().map(|&x| parse(x.as_bytes(), &mut identifiers).unwrap()).collect();

        assert_eq!(
            tokens,
            vec![
                Token::Noun(Noun::Identifier(0)),
                Token::Noun(Noun::Identifier(1)),
                Token::Noun(Noun::Identifier(0)),
                Token::Noun(Noun::Identifier(2)),
                Token::Noun(Noun::Identifier(3)),
                Token::Noun(Noun::Identifier(2)),
                Token::Noun(Noun::Identifier(1)),
                Token::Noun(Noun::Identifier(0))
            ]
        )
    }

    #[test]
    fn parse_mixed() {
        let string = "baba and keke not on _ baba and 4 me is keke baba empty aaa";
        
        let mut identifiers = HashMap::new();
        let words: Vec<&str> = string.split_ascii_whitespace().collect();
        let tokens: Vec<Token> = words.iter().map(|&x| parse(x.as_bytes(), &mut identifiers).unwrap()).collect();

        assert_eq!(tokens, 
            vec![
                Token::Noun(Noun::Identifier(0)),
                Token::And,
                Token::Noun(Noun::Identifier(1)),
                Token::Not,
                Token::Conditional(Conditional::On),
                Token::Noun(Noun::Identifier(2)),
                Token::Noun(Noun::Identifier(0)),
                Token::And,
                Token::Noun(Noun::Identifier(3)),
                Token::Noun(Noun::Identifier(4)),
                Token::Verb(Verb::Is),
                Token::Noun(Noun::Identifier(1)),
                Token::Noun(Noun::Identifier(0)),
                Token::Noun(Noun::Empty),
                Token::Noun(Noun::Identifier(5))
            ]
        )
    }
}