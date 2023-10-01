use crate::evaluator::macro_expansion::{define_macros, expand_macro};
use crate::{
    ast::traits::AsNode, evaluator::environment::Environment, evaluator::eval::eval, lexer::Lexer,
    parser::Parser,
};
use std::io::{self, Write};
use std::{cell::RefCell, rc::Rc};

const PROMPT: &str = ">> ";

pub fn start<W: Write>(mut output: W) -> io::Result<()> {
    let env = Rc::new(RefCell::new(Environment::new()));
    let macro_env = Rc::new(RefCell::new(Environment::new()));
    loop {
        let mut line = String::new();
        write!(output, "{}", PROMPT)?;
        io::Write::flush(&mut io::stdout())?;

        io::stdin().read_line(&mut line)?;
        let lexer = Lexer::new(line);
        let mut parser = Parser::new(lexer);
        let mut program = parser.parse_program();

        if !parser.error_messages.is_empty() {
            print_parser_errors(&mut output, &parser.error_messages)?;
            continue;
        }
        define_macros(&mut program, Rc::clone(&macro_env));
        expand_macro(&mut program, Rc::clone(&macro_env));
        let evaluated = eval(program.as_node(), Rc::clone(&env));
        writeln!(output, "{}", evaluated.inspect())?;
    }
}

fn print_parser_errors<W: Write>(output: &mut W, errors: &[String]) -> io::Result<()> {
    writeln!(output, "Woops! We ran into some monkey bussiness here!")?;
    writeln!(output, " parser errors:")?;
    for error in errors {
        writeln!(output, "{}", error)?;
    }
    Ok(())
}
