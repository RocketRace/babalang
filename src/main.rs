mod error_handler;

mod token;
mod lexer;

mod statement;
mod statement_parser;

mod instruction;
mod ast;

use std::env;

/// Babalang interpreter
fn main() -> std::io::Result<()> {
    // Get path of source file
    let mut args = env::args();
    let _program_path = args.next().unwrap();
    let file_path = match args.next() {
        Some(x) => x,
        None => {
            error_handler::throw_error_str(
                error_handler::ErrorType::FileError,
                "File not provided."
            );
            panic!() // necessary for the match arms to match 
        }
    };

    // Tokenize the source file and return a vector of tokens
    let (tokens, identifiers) = lexer::tokenize(&file_path);
    println!("Successfully tokenized program at `{}`.", file_path);

    // A vector of Statements (e.g. BABA IS YOU)
    let statements = statement_parser::parse(&tokens);
    println!("Successfully parsed program into statements.");

    let ast = ast::parse(&statements);
    println!("Successfully parsed statements into an AST.");
    println!("{:?}", ast);

    // Done
    Ok(())
}