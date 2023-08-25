use crate::{lexer::Lexer, token::TokenType};
use std::io::{self, Write};

const PROMPT: &str = ">> ";

pub fn start<W: Write>(mut output: W) -> io::Result<()> {
    let mut line = String::new();

    write!(output, "{}", PROMPT)?;
    io::Write::flush(&mut io::stdout())?;

    io::stdin().read_line(&mut line)?;
    let mut lexer = Lexer::new(line);

    loop {
        let token = lexer.next_token();
        writeln!(output, "{:?}", token)?;

        if token.token_type == TokenType::EOF {
            break;
        }
    }

    Ok(())
}
