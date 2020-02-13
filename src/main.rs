mod error_handler;
mod lexer;
mod statement;
mod statement_parser;
mod token;

use std::env;

/// Babalang interpreter
fn main() -> std::io::Result<()> {
    // Get path of source file
    let mut args = env::args();
    let _program_path = args.next().unwrap();
    let file_path = args.next().unwrap();

    // Tokenize the source file and return a vector of tokens
    let (tokens, identifiers) = lexer::tokenize(&file_path);
    println!("Successfully tokenized program at `{}`.", file_path);

    // A vector of Statements (e.g. BABA IS YOU)
    let statements = statement_parser::parse(&tokens);
    println!("Successfully parsed program into statements.");

    // Done
    Ok(())
}