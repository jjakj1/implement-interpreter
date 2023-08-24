use crate::{lexer::Lexer, token::EOF};
use std::io;

const PROMPT: &str = ">> ";

pub fn start() -> io::Result<()> {
    print!("{}", PROMPT);
    io::Write::flush(&mut io::stdout()).expect("flash failed");

    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input)?;

    let mut lexer = Lexer::new(user_input);

    let mut token = lexer.next_token();

    while token.token_type != EOF {
        println!("{:?}", token);
        token = lexer.next_token();
    }

    Ok(())
}
