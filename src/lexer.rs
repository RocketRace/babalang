use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

use crate::token::{Token, parse};
use crate::error_handler::{ErrorType, throw_error};

/// The simple internal state of the lexer.
/// 
/// Dictates whether the lexer is reading a word or a separator.
enum State {
    Word,
    Separator,
    MaybeComment,
    Comment
}

/// Tokenizes a Baba source file from the given path.
/// Returns a vector of tokens if tokenization is successful,
/// as well as a hashmap of the identifiers encountered.
/// 
/// # Arguments
/// 
/// * `path` - Source path for the program being tokenized.
/// 
/// # Return
/// 
/// Returns a tuple containing:
/// 
/// * `Vec<Token>` - The tokens parsed from the file.
/// 
/// * `HashMap<String, usize>` - A mapping between identifiers (e.g. "baba")
/// and their corresponding IDs.
pub fn tokenize(path: Option<String>, source: Option<&mut Vec<u8>>) -> (Vec<Token>, HashMap<usize, String>) {
    let mut buffer = Vec::new();
    
    if let Some(p) = path {
        let mut file = match File::open(&p) {
            Ok(f) => f,
            Err(_) => {
                throw_error(
                    ErrorType::FileError, 
                    format!("Could not open file at `{}`", p),
                    None
                );
                panic!() // necessary for match arms to match
            }
        };
        file.read_to_end(&mut buffer).unwrap();
    }
    else {
        if let Some(bytes) = source {
            buffer.append(bytes);
        }
    }

    let mut out: Vec<Token> = Vec::new();
    let mut identifiers: HashMap<usize, String> = HashMap::new();
    identifiers.insert(0, "empty".to_string());
    identifiers.insert(1, "level".to_string());
    identifiers.insert(2, "image".to_string());
    let mut state = State::Separator;
    let mut word_start = 0;

    for (i, &byte) in buffer.iter().enumerate() {
        let c = char::from(byte)
            .to_lowercase().next()
            .unwrap();
        // Simple state machine
        match state {
            // Not in a word
            State::Separator => {
                // Begin new word
                if c.is_ascii_alphanumeric() || c == '_' {
                    state = State::Word;
                }
                else if c == '/' {
                    state = State::MaybeComment;
                    word_start += 1;
                }
                else {
                    // The current word won't start here yet
                    word_start += 1;
                }
            },
            // In a word
            State::Word => {
                // Continue existing word
                if c.is_ascii_alphanumeric() || c == '_' {
                    state = State::Word;
                }
                // Parse the current word into a token
                else {
                    state = State::Separator;
                    let word = &buffer[word_start..i];
                    // Empty strings aren't tokens (we should never encounter any)
                    if let Some(token) = parse(word, &mut identifiers) {
                        out.push(token);
                    }
                    else {
                        throw_error(
                            ErrorType::LexerError,
                            format!("Failed to parse input: {:?}", &word),
                            None
                        );
                    };
                    word_start = i + 1;
                }
            },
            // This might be a comment? ("//")
            State::MaybeComment => {
                if c.is_ascii_alphanumeric() || c == '_' {
                    state = State::Word;
                }
                else if c == '/' {
                    state = State::Comment;
                    word_start += 1;
                }
                else {
                    // The current word won't start here yet
                    word_start += 1;
                }
            }
            // This certainly is a comment. ("//")
            State::Comment => {
                if c == '\n' || c == '\r' {
                    state = State::Separator;
                    word_start += 1;
                }
                else {
                    word_start += 1;
                }
            }
        }
    }
    // Account for EOF
    if let State::Word = state {
        let word = &buffer[word_start..];
        if let Some(token) = parse(word, &mut identifiers) {
            out.push(token);
        }
        else {
            throw_error(
                ErrorType::LexerError,
                format!("Failed to parse input: {:?}", &word),
                None
            );
        };
    }
    let output = out.to_owned();
    let id = identifiers.to_owned();

    (output, id)
}

#[cfg(test)]
mod tests {
    use crate::lexer::tokenize;
    use crate::token::{Token, Noun, Verb};

    #[test]
    fn tokenize_alnum() {
        let path = String::from("tests/tokenize_alnum.baba");
        let (tokens, _identifiers) = tokenize(Some(path), None);

        assert_eq!(
            tokens,
            vec![
                Token::Noun(Noun::Identifier(3)),
                Token::Noun(Noun::Identifier(3)),
                Token::Noun(Noun::Identifier(3)),
                Token::Noun(Noun::Identifier(3)),
                Token::Verb(Verb::Is),
                Token::Verb(Verb::Is),
                Token::Verb(Verb::Is),
                Token::Verb(Verb::Is),
                Token::Noun(Noun::Identifier(4)),
                Token::Noun(Noun::Identifier(4)),
                Token::Noun(Noun::Identifier(4)),
                Token::Noun(Noun::Identifier(4)),
                Token::Noun(Noun::Empty),
                Token::Noun(Noun::Empty),
                Token::Noun(Noun::Empty),
                Token::Noun(Noun::Identifier(5)),
                Token::Noun(Noun::Identifier(6)),
                Token::Noun(Noun::Identifier(7)),
                Token::Noun(Noun::Identifier(8))
            ]
        )
    }
}