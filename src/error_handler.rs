use std::process::exit;
/// Dictates the source of the error.
#[derive(Debug)]
pub enum ErrorType {
    FileError,
    LexerError,
    StatementParserError,
    InstructionParserError,
    InstructionValidationError,
}

/// Throws an exception and panics the current thread.
/// 
/// # Arguments
/// 
/// * `error_type` - An enum variant that dictates the type of error thrown.
/// 
/// * `error_message` - The message to display on panic.
pub fn throw_error_str(error_type: ErrorType, error_message: &str) {
    println!("{:?}: {}", error_type, error_message);
    exit(1);
}

/// Throws an exception and panics the current thread.
/// 
/// # Arguments
/// 
/// * `error_type` - An enum variant that dictates the type of error thrown.
/// 
/// * `error_message` - The message to display on panic.
pub fn throw_error(error_type: ErrorType, error_message: String) {
    println!("{:?}: {}", error_type, error_message);
    exit(1);
}