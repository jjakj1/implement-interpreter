use crate::{
    ast::traits::AsNode, environment::Environment, evaluator::eval, lexer::Lexer, parser::Parser,
};
use std::io::{self, Write};
use std::{cell::RefCell, rc::Rc};

const PROMPT: &str = ">> ";

pub fn start<W: Write>(mut output: W) -> io::Result<()> {
    loop {
        let mut line = String::new();
        write!(output, "{}", PROMPT)?;
        io::Write::flush(&mut io::stdout())?;

        io::stdin().read_line(&mut line)?;
        let lexer = Lexer::new(line);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        if !parser.error_messages.is_empty() {
            print_parser_errors(&mut output, &parser.error_messages)?;
            continue;
        }

        let env = Environment::new();
        let evaluated = eval(program.as_node(), Rc::new(RefCell::new(env)));
        if let Some(evaluated) = evaluated {
            writeln!(output, "{}", evaluated.inspect())?;
        }
    }
}

fn print_parser_errors<W: Write>(output: &mut W, errors: &Vec<String>) -> io::Result<()> {
    writeln!(output, "Woops! We ran into some monkey bussiness here!")?;
    writeln!(output, " parser errors:")?;
    for error in errors {
        writeln!(output, "{}", error)?;
    }
    Ok(())
}
