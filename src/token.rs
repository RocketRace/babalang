use std::iter::FromIterator;

// Valid tokens
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Noun {
    All,
    Identifier(String)
}

#[derive(Debug)]
#[derive(PartialEq)]
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

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Property {
    You,
    Move,
    Text
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Prefix {
    Lonely,
    Idle,
    Powered
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Conditional {
    On,
    Near,
    Facing,
    Without
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum LexToken {
    NounToken(Noun),
    VerbToken(Verb),
    PropertyToken(Property),
    PrefixToken(Prefix),
    NotToken,
    AndToken,
    ConditionalToken(Conditional)
}

/// Allows for parsing tokens from char slices.
pub trait Token {
    fn parse(buffer: &[char]) -> Option<LexToken>;
}

impl Token for LexToken {
    /// Parses a char slice into the associated token.
    /// 
    /// Returns None if the slice is empty.
    fn parse(buffer: &[char]) -> Option<LexToken> {
        if buffer.len() == 0 {
            None
        }
        else {
            let id = String::from_iter(buffer);

            let token = match id.as_str() {
                // Noun keywords
                "all" => LexToken::NounToken(Noun::All),
                // Verb keywords
                "is" => LexToken::VerbToken(Verb::Is),
                "has" => LexToken::VerbToken(Verb::Has),
                "make" => LexToken::VerbToken(Verb::Make),
                "mimic" => LexToken::VerbToken(Verb::Mimic),
                // Property keywords
                "you" => LexToken::PropertyToken(Property::You),
                "move" => LexToken::PropertyToken(Property::Move),
                "text" => LexToken::PropertyToken(Property::Text),
                // Prefix keywords
                // TODO: implement
                // "And"
                "and" => LexToken::AndToken,
                // "Not"
                "not" => LexToken::NotToken,
                // "Conditional" keywords
                // TODO: implement
                // Everything else (identifiers)
                _ => LexToken::NounToken(Noun::Identifier(id))
            };
            Some(token)
        }
    }
}