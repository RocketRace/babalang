use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

use crate::token::{LexToken, parse};
use crate::error_handler::{ErrorType, throw_error};

/// The simple internal state of the lexer.
/// 
/// Dictates whether the lexer is reading a word or a separator.
enum State {
    Word,
    Separator
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
/// * `Vec<LexToken>` - The tokens parsed from the file.
/// 
/// * `HashMap<String, usize>` - A mapping between identifiers (e.g. "baba")
/// and their corresponding IDs.
pub fn tokenize(path: &str) -> (Vec<LexToken>, HashMap<String, usize>) {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => {
            throw_error(
                ErrorType::FileError, 
                &format!("Could not open file at `{}`", path)
            );
            panic!() // necessary for match arms to match
        }
    };

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let mut out: Vec<LexToken> = Vec::new();
    let mut identifiers: HashMap<String, usize> = HashMap::new();
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
                            &format!("Failed to parse input: {:?}", &word)
                        );
                    };
                    word_start = i + 1;
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
                &format!("Failed to parse input: {:?}", &word)
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
    use crate::token::{LexToken, NounToken, VerbToken, PropertyToken, ConditionalToken};

    #[test]
    fn tokenize_alnum() {
        let path = "tests/tokenize_alnum.baba";
        let (tokens, identifiers) = tokenize(path);

        assert_eq!(
            tokens,
            vec![
                LexToken::Noun(NounToken::Identifier(0)),
                LexToken::Noun(NounToken::Identifier(0)),
                LexToken::Noun(NounToken::Identifier(0)),
                LexToken::Noun(NounToken::Identifier(0)),
                LexToken::Verb(VerbToken::Is),
                LexToken::Verb(VerbToken::Is),
                LexToken::Verb(VerbToken::Is),
                LexToken::Verb(VerbToken::Is),
                LexToken::Noun(NounToken::Identifier(1)),
                LexToken::Noun(NounToken::Identifier(1)),
                LexToken::Noun(NounToken::Identifier(1)),
                LexToken::Noun(NounToken::Identifier(1)),
                LexToken::Noun(NounToken::Empty),
                LexToken::Noun(NounToken::Empty),
                LexToken::Noun(NounToken::Empty),
                LexToken::Noun(NounToken::Identifier(2)),
                LexToken::Noun(NounToken::Identifier(3)),
                LexToken::Noun(NounToken::Identifier(4)),
                LexToken::Noun(NounToken::Identifier(5))
            ]
        )
    }
}