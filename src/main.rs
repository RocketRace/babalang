mod lexer;
mod token;

use std::env;

/// Babalang interpreter
fn main() -> std::io::Result<()> {
    // Get path of source file
    let mut args = env::args();
    let _program_path = args.next().unwrap();
    let file_path = args.next().unwrap();

    // Tokenize the source file and return a slice of tokens
    let token_stream = lexer::tokenize(&file_path);
    match token_stream {
        Ok(_) => println!("Successfully tokenized program."),
        Err(_) => panic!()
    };

    // Done
    Ok(())
}