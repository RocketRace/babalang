use std::fs::File;
use std::io::Read;

use crate::token::{LexToken, Token};

/// The simple internal state of the lexer.
/// 
/// Dictates whether the lexer is reading a word or a separator.
enum State {
    Word,
    Separator
}

/// Tokenizes a Baba source file from the given path.
/// 
/// Returns a vector of tokens if tokenization is successful.
pub fn tokenize(path: &str) -> Result<Vec<LexToken>, &'static str> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => panic!()
    };

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let mut out: Vec<LexToken> = Vec::new();
    let mut word: Vec<char> = Vec::new();

    let mut state = State::Separator;

    for byte in buffer {
        let c = char::from(byte)
            .to_lowercase().next()
            .unwrap();
        // simple state machine
        match state {
            // not in a word
            State::Separator => {
                // begin new word
                if c.is_ascii_alphanumeric() || c == '_' {
                    state = State::Word;
                    word.push(c);
                }
            },
            // in a word
            State::Word => {
                // continue existing word
                if c.is_ascii_alphanumeric() || c == '_' {
                    word.push(c);
                }
                // parse the current word into a token
                else {
                    // empty strings aren't tokens
                    let token = match LexToken::parse(&word){
                        Some(t) => t,
                        None => panic!()
                    };
                    out.push(token);
                    word.clear();
                }
            }
        }
    }
    // Account for EOF
    if word.len() > 0 {
        let token = match LexToken::parse(&word){
            Some(t) => t,
            None => panic!()
        };
        out.push(token);
        word.clear();
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use crate::token::{Noun, Verb, Property};
    use super::*;

    #[test]
    fn test_tokenize() {
        assert_eq!(
            // File to be tokenized
            tokenize("test.baba").unwrap(), 
            // Expected result
            vec![
                LexToken::NounToken(Noun::Identifier(String::from("baba"))), 
                LexToken::VerbToken(Verb::Is), 
                LexToken::PropertyToken(Property::You),
                LexToken::NounToken(Noun::Identifier(String::from("baba"))), 
                LexToken::VerbToken(Verb::Is), 
                LexToken::PropertyToken(Property::Move),
            ]
        );
    }
}