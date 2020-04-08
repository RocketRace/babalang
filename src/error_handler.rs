use std::process::exit;
use std::collections::HashMap;

use std::io::{stderr, Write};

/// Dictates the source of the error.
#[derive(Debug)]
pub enum ErrorType {
    FileError,
    LexerError,
    StatementParserError,
    InstructionParserError,
    InstructionValidationError,
    RuntimeError,
    ObjectNotDefinedError,
    ObjectAlreadyDefinedError,
    TypeError,
    ArgumentError,
    ConditionError,
}

/// Throws an exception and panics the current thread.
/// 
/// # Arguments
/// 
/// * `error_type` - An enum variant that dictates the type of error thrown.
/// 
/// * `error_message` - The message to display on panic.
pub fn throw_error_str(error_type: ErrorType, error_message: &str) {
    stderr().write(format!("{:?}: {}\n", error_type, error_message).as_bytes()).unwrap();
    exit(1);
}

/// Throws an exception and panics the current thread.
/// 
/// # Arguments
/// 
/// * `error_type` - An enum variant that dictates the type of error thrown.
/// 
/// * `error_message` - The message to display on panic.
pub fn throw_error(
    error_type: ErrorType, 
    error_message: String, 
    identifers: Option<(&[usize], &HashMap<usize, String>)>
) {
    let mut handle = stderr();
    handle.write(format!("{:?}: {}\n", error_type, error_message).as_bytes()).unwrap();
    if let Some((used, ids)) = identifers {
        handle.write("[Identifiers: ".as_bytes()).unwrap();
        for (i, id) in used.iter().enumerate() {
            // Unwrap is used since errors should only be raised for existing values
            if i == 0 {
                handle.write(format!("{} = \"{}\"", id, ids.get(id).unwrap()).as_bytes()).unwrap();
            }
            else {
                handle.write(format!(", {} = \"{}\"", id, ids.get(id).unwrap()).as_bytes()).unwrap();
            }
        }
        handle.write("]\n".as_bytes()).unwrap();
    }
    exit(1);
}