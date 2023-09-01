use lazy_static::lazy_static;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{
    BlockStatement, Boolean, CallExpression, Expression, ExpressionStatement, FunctionLiteral,
    Identifier, IfExpression, InfixExpression, IntegerLiteral, LetStatement, PrefixExpression,
    Program, ReturnStatement, Statement,
};
use crate::token::TokenType;
use crate::{lexer::Lexer, token::Token};

type PrefixParseFn = dyn Fn(&mut Parser) -> Result<Box<dyn Expression>, String>;
type InfixParseFn = dyn Fn(&mut Parser, Box<dyn Expression>) -> Result<Box<dyn Expression>, String>;

pub struct Parser {
    lexer: Lexer,
    current_token: Option<Token>,
    peek_token: Option<Token>,
    pub error_messages: Vec<String>,
    prefix_parse_fns: HashMap<TokenType, Rc<PrefixParseFn>>,
    infix_parse_fns: HashMap<TokenType, Rc<InfixParseFn>>,
}

#[derive(Debug, Clone, Copy)]
enum ExpressionPrecedence {
    Lowest = 1,      // 标识符
    Equals = 2,      // ==
    LessGreater = 3, // < or >
    Sum = 4,         // +
    Product = 5,     // *
    Prefix = 6,      // -x or !x
    Call = 7,        // myFunction(x)
}

lazy_static! {
    static ref PRECEDENCES: HashMap<TokenType, ExpressionPrecedence> = HashMap::from([
        (TokenType::Equal, ExpressionPrecedence::Equals),
        (TokenType::NotEqual, ExpressionPrecedence::Equals),
        (TokenType::LessThan, ExpressionPrecedence::LessGreater),
        (TokenType::GreaterThan, ExpressionPrecedence::LessGreater),
        (TokenType::Plus, ExpressionPrecedence::Sum),
        (TokenType::Minus, ExpressionPrecedence::Sum),
        (TokenType::Slash, ExpressionPrecedence::Product),
        (TokenType::Asterisk, ExpressionPrecedence::Product),
        (TokenType::LeftParen, ExpressionPrecedence::Call),
    ]);
}

impl Parser {
    pub fn new(lexer: Lexer) -> Parser {
        let mut parser = Parser {
            lexer,
            current_token: None,
            peek_token: None,
            error_messages: vec![],
            prefix_parse_fns: HashMap::new(),
            infix_parse_fns: HashMap::new(),
        };
        parser.register_prefix(TokenType::Ident, Rc::new(Parser::parse_identifier));
        parser.register_prefix(TokenType::Int, Rc::new(Parser::parse_integer_literal));
        parser.register_prefix(TokenType::Bang, Rc::new(Parser::parse_prefix_expression));
        parser.register_prefix(TokenType::Minus, Rc::new(Parser::parse_prefix_expression));
        parser.register_prefix(TokenType::True, Rc::new(Parser::parse_boolean));
        parser.register_prefix(TokenType::False, Rc::new(Parser::parse_boolean));
        parser.register_prefix(
            TokenType::LeftParen,
            Rc::new(Parser::parse_grouped_expression),
        );
        parser.register_prefix(TokenType::If, Rc::new(Parser::parse_if_expression));
        parser.register_prefix(TokenType::Function, Rc::new(Parser::parse_function_literal));

        parser.register_infix(TokenType::Plus, Rc::new(Parser::parse_infix_expression));
        parser.register_infix(TokenType::Minus, Rc::new(Parser::parse_infix_expression));
        parser.register_infix(TokenType::Slash, Rc::new(Parser::parse_infix_expression));
        parser.register_infix(TokenType::Asterisk, Rc::new(Parser::parse_infix_expression));
        parser.register_infix(TokenType::Equal, Rc::new(Parser::parse_infix_expression));
        parser.register_infix(TokenType::NotEqual, Rc::new(Parser::parse_infix_expression));
        parser.register_infix(TokenType::LessThan, Rc::new(Parser::parse_infix_expression));
        parser.register_infix(
            TokenType::GreaterThan,
            Rc::new(Parser::parse_infix_expression),
        );
        parser.register_infix(TokenType::LeftParen, Rc::new(Parser::parse_call_expression));
        parser.next_token();
        parser.next_token();
        parser
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.take();
        self.peek_token = Some(self.lexer.next_token());
    }

    pub fn parse_program(&mut self) -> Program {
        // TODO: improve error type
        let mut program = Program { statements: vec![] };

        // TODO: 这个地方是不是只能 clone，take 的话原来的值就变成 None 了
        loop {
            if let Some(token) = self.current_token.clone() {
                if token.token_type != TokenType::EOF {
                    self.parse_statement().map_or_else(
                        |error_message| {
                            self.error_messages.push(error_message);
                        },
                        |statement| {
                            program.statements.push(statement);
                        },
                    );
                    self.next_token();
                } else {
                    break;
                }
            }
        }
        program
    }

    fn parse_statement(&mut self) -> Result<Box<dyn Statement>, String> {
        let current_token_type = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .token_type;
        match current_token_type {
            TokenType::Let => self.parse_let_statement(),
            TokenType::Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Box<dyn Statement>, String> {
        let let_token = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .clone();

        self.expect_peek_token(TokenType::Ident)?;
        let iden = self.current_token.as_ref().unwrap().clone();
        let identifier = Identifier {
            token: iden.clone(),
            value: iden.literal,
        };

        self.expect_peek_token(TokenType::Assign)?;
        self.next_token();

        let let_statement = LetStatement {
            token: let_token,
            name: identifier,
            value: self.parse_expression(ExpressionPrecedence::Lowest)?,
        };
        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }
        Ok(Box::new(let_statement))
    }

    fn parse_return_statement(&mut self) -> Result<Box<dyn Statement>, String> {
        let return_token = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .clone();

        self.next_token();

        let return_value = self.parse_expression(ExpressionPrecedence::Lowest)?;

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Ok(Box::new(ReturnStatement {
            token: return_token,
            return_value,
        }))
    }

    fn parse_expression_statement(&mut self) -> Result<Box<dyn Statement>, String> {
        let token = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .clone();
        let statement = Ok(Box::new(ExpressionStatement {
            token,
            expression: self.parse_expression(ExpressionPrecedence::Lowest)?,
        }) as Box<dyn Statement>);

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        statement
    }

    fn parse_expression(
        &mut self,
        precedence: ExpressionPrecedence,
    ) -> Result<Box<dyn Expression>, String> {
        let token_type = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .token_type;
        let prefix_parse_function = self
            .prefix_parse_fns
            .get(&token_type) // 感觉这里加了 `as_ref` 就变成了对内部 token 的引用了
            .ok_or(format!(
                "No prefix parse function for {:?} found",
                token_type
            ))?;
        let mut left_expression = Rc::clone(prefix_parse_function)(self)?;

        while !self.peek_token_is(TokenType::Semicolon)
            && (precedence as i32) < (self.peek_precedence() as i32)
        {
            let peek_token_type = self
                .peek_token
                .as_ref()
                .ok_or("Peek token is None")?
                .token_type;
            match self.infix_parse_fns.get(&peek_token_type) {
                Some(infix_parse_fn) => {
                    let infix_parse_fn = Rc::clone(infix_parse_fn);
                    self.next_token();
                    left_expression = infix_parse_fn(self, left_expression)?;
                }
                None => {
                    return Ok(left_expression);
                }
            }
        }
        Ok(left_expression)
    }

    fn parse_identifier(&mut self) -> Result<Box<dyn Expression>, String> {
        let token = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .clone();
        Ok(Box::new(Identifier {
            token: token.clone(),
            value: token.literal,
        }) as Box<dyn Expression>)
    }

    fn parse_integer_literal(&mut self) -> Result<Box<dyn Expression>, String> {
        let token = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .clone();
        Ok(Box::new(IntegerLiteral {
            token: token.clone(),
            value: token
                .literal
                .parse()
                .map_err(|error| format!("{:?}", error))?, // TODO
        }) as Box<dyn Expression>)
    }

    fn parse_prefix_expression(&mut self) -> Result<Box<dyn Expression>, String> {
        let token = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .clone();
        self.next_token(); // 只有需要继续解析才需要调用 next_token
        Ok(Box::new(PrefixExpression {
            token: token.clone(),
            operator: token.literal,
            right: self.parse_expression(ExpressionPrecedence::Prefix)?,
        }) as Box<dyn Expression>)
    }

    fn parse_infix_expression(
        &mut self,
        left: Box<dyn Expression>,
    ) -> Result<Box<dyn Expression>, String> {
        let token = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .clone();
        let precedence = self.current_precedence();
        self.next_token();
        Ok(Box::new(InfixExpression {
            token: token.clone(),
            left,
            operator: token.literal,
            right: self.parse_expression(precedence)?,
        }) as Box<dyn Expression>)
    }

    fn parse_grouped_expression(&mut self) -> Result<Box<dyn Expression>, String> {
        self.next_token();

        let expression = self.parse_expression(ExpressionPrecedence::Lowest)?;
        self.expect_peek_token(TokenType::RightParen)?;
        Ok(expression)
    }

    fn parse_boolean(&mut self) -> Result<Box<dyn Expression>, String> {
        let token = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .clone();
        Ok(Box::new(Boolean {
            token,
            value: self.current_token_is(TokenType::True),
        }))
    }

    fn parse_if_expression(&mut self) -> Result<Box<dyn Expression>, String> {
        let token = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .clone();
        self.expect_peek_token(TokenType::LeftParen)?;
        self.next_token();
        let condition = self.parse_expression(ExpressionPrecedence::Lowest)?;
        self.expect_peek_token(TokenType::RightParen)?;
        self.expect_peek_token(TokenType::LeftBrace)?;
        let consequence = self.parse_block_statement()?;
        let mut if_expression = IfExpression {
            token,
            condition,
            consequence,
            alternative: None,
        };

        if self.peek_token_is(TokenType::Else) {
            self.next_token();
            self.expect_peek_token(TokenType::LeftBrace)?;
            if_expression.alternative = Some(self.parse_block_statement()?);
        }

        Ok(Box::new(if_expression))
    }

    fn parse_function_literal(&mut self) -> Result<Box<dyn Expression>, String> {
        let token = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .clone();
        self.expect_peek_token(TokenType::LeftParen)?;
        let parameters = self.parse_function_parameters()?;
        self.expect_peek_token(TokenType::LeftBrace)?;
        Ok(Box::new(FunctionLiteral {
            token,
            parameters,
            body: self.parse_block_statement()?,
        }))
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<Identifier>, String> {
        let mut idents = Vec::new();
        self.next_token();
        if self.current_token_is(TokenType::RightParen) {
            return Ok(idents);
        }

        loop {
            let token = self
                .current_token
                .as_ref()
                .ok_or("Current token is None")?
                .clone();
            let identifier = Identifier {
                token: token.clone(),
                value: token.literal,
            };
            idents.push(identifier);
            if self.peek_token_is(TokenType::Comma) {
                self.next_token();
                self.next_token();
            } else {
                break;
            }
        }
        self.expect_peek_token(TokenType::RightParen)?;
        Ok(idents)
    }

    fn parse_call_expression(
        &mut self,
        left: Box<dyn Expression>,
    ) -> Result<Box<dyn Expression>, String> {
        let token = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .clone();
        let arguments = self.parse_call_arguments()?;
        Ok(Box::new(CallExpression {
            token,
            function: left,
            arguments,
        }))
    }

    fn parse_call_arguments(&mut self) -> Result<Vec<Box<dyn Expression>>, String> {
        let mut args = Vec::new();
        self.next_token();
        if self.current_token_is(TokenType::RightParen) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expression(ExpressionPrecedence::Lowest)?);

            if self.peek_token_is(TokenType::Comma) {
                self.next_token();
                self.next_token();
            } else {
                break;
            }
        }

        self.expect_peek_token(TokenType::RightParen)?;
        Ok(args)
    }

    fn parse_block_statement(&mut self) -> Result<BlockStatement, String> {
        let token = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .clone();
        let mut statements = vec![];
        self.next_token();
        while !self.current_token_is(TokenType::RightBrace)
            && !self.current_token_is(TokenType::EOF)
        {
            if let Ok(statement) = self.parse_statement() {
                statements.push(statement);
            }
            self.next_token();
        }
        Ok(BlockStatement { token, statements })
    }

    fn current_token_is(&self, token_type: TokenType) -> bool {
        self.current_token
            .as_ref()
            .map_or(false, |token| token.token_type == token_type)
    }

    fn peek_token_is(&self, token_type: TokenType) -> bool {
        self.peek_token
            .as_ref()
            .map_or(false, |token| token.token_type == token_type)
    }

    fn expect_peek_token(&mut self, token_type: TokenType) -> Result<(), String> {
        if self.peek_token_is(token_type) {
            self.next_token();
            Ok(())
        } else {
            Err(format!(
                "expected next token to be {:?}, got {:?} instead",
                token_type,
                self.peek_token
                    .as_ref()
                    .map_or(TokenType::Illegal, |token| token.token_type)
            ))
        }
    }

    fn register_prefix(&mut self, token_type: TokenType, fn_ptr: Rc<PrefixParseFn>) {
        self.prefix_parse_fns.insert(token_type, fn_ptr);
    }

    fn register_infix(&mut self, token_type: TokenType, fn_ptr: Rc<InfixParseFn>) {
        self.infix_parse_fns.insert(token_type, fn_ptr);
    }

    fn peek_precedence(&self) -> ExpressionPrecedence {
        self.peek_token
            .as_ref()
            .and_then(|token| PRECEDENCES.get(&token.token_type))
            .unwrap_or(&ExpressionPrecedence::Lowest)
            .to_owned()
    }

    fn current_precedence(&self) -> ExpressionPrecedence {
        self.current_token
            .as_ref()
            .and_then(|token| PRECEDENCES.get(&token.token_type))
            .unwrap_or(&ExpressionPrecedence::Lowest)
            .to_owned()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            Boolean, CallExpression, Expression, ExpressionStatement, FunctionLiteral, Identifier,
            IfExpression, InfixExpression, IntegerLiteral, LetStatement, Node, PrefixExpression,
            Program, ReturnStatement,
        },
        lexer::Lexer,
    };

    use super::Parser;

    #[test]
    fn test_let_statements() {
        struct LetTest {
            input: String,
            expected_identifier: String,
            expected_value: String,
        }

        let tests = [
            LetTest {
                input: "let x = 5;".to_owned(),
                expected_identifier: "x".to_owned(),
                expected_value: "5".to_owned(),
            },
            LetTest {
                input: "let y = true;".to_owned(),
                expected_identifier: "y".to_owned(),
                expected_value: "true".to_owned(),
            },
            LetTest {
                input: "let foobar = y;".to_owned(),
                expected_identifier: "foobar".to_owned(),
                expected_value: "y".to_owned(),
            },
        ];

        for test in tests {
            let lexer = Lexer::new(test.input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();
            for err in parser.error_messages {
                eprintln!("{}", err);
            }
            assert_eq!(program.statements.len(), 1);
            let statement = program
                .statements
                .first()
                .and_then(|statement| statement.as_any().downcast_ref::<LetStatement>())
                .unwrap();
            assert_eq!(statement.token_literal(), "let");
            assert_eq!(statement.name.string(), test.expected_identifier);
            assert_eq!(statement.value.string(), test.expected_value);
        }
    }

    // #[test]
    // fn test_illegal_let_statements() {
    //     let illegal_input = "
    //         let x 5;
    //         let = 10;
    //         let = 838383;";

    //     let lexer = Lexer::new(illegal_input.to_owned());
    //     let mut parser = Parser::new(lexer);
    //     let _ = parser.parse_program();
    //     assert_eq!(parser.error_messages.len(), 3);

    //     for message in parser.error_messages.iter() {
    //         eprintln!("parser error: {}", message);
    //     }
    // }

    #[test]
    fn test_return_statements() {
        let input = "
            return 5;
            return 10;
            return 993;";

        let lexer = Lexer::new(input.to_owned());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert_eq!(program.statements.len(), 3);
        for statment in program.statements.iter() {
            let return_statement = statment
                .as_any()
                .downcast_ref::<ReturnStatement>()
                .expect("statement is not `ReturnStatment` type");
            assert_eq!(return_statement.token_literal(), "return")
        }
    }

    #[test]
    fn test_indentifier_expression() {
        let input = "foobar";

        let lexer = Lexer::new(input.to_owned());
        let mut parse = Parser::new(lexer);
        let program = parse.parse_program();
        assert_eq!(program.statements.len(), 1);

        let expression = get_first_expression::<Identifier>(&program);

        assert_eq!(expression.value, "foobar");
        assert_eq!(expression.token_literal(), "foobar");
    }

    #[test]
    fn test_integer_literal_expression() {
        let input = "5;";

        let lexer = Lexer::new(input.to_owned());
        let mut parse = Parser::new(lexer);
        let program = parse.parse_program();
        assert_eq!(program.statements.len(), 1);

        let integer_literal = get_first_expression::<IntegerLiteral>(&program);

        assert_eq!(integer_literal.value, 5);
        assert_eq!(integer_literal.token_literal(), "5");
    }

    #[test]
    fn test_prefix_expression() {
        trait PrefixTest {
            fn input(&self) -> String;
            fn test_expression(&self, program: &Program);
        }

        struct IntegerPrefixTest {
            input: String,
            operator: String,
            integer_value: i64,
        }

        impl PrefixTest for IntegerPrefixTest {
            fn input(&self) -> String {
                self.input.clone()
            }

            fn test_expression(&self, program: &Program) {
                let expression = get_first_expression::<PrefixExpression>(program);
                assert_eq!(expression.operator, self.operator);
                test_integer_literal(expression.right.as_ref(), self.integer_value);
            }
        }

        struct BooleanPrefixTest {
            input: String,
            operator: String,
            boolean_value: bool,
        }

        impl PrefixTest for BooleanPrefixTest {
            fn input(&self) -> String {
                self.input.clone()
            }

            fn test_expression(&self, program: &Program) {
                let expression = get_first_expression::<PrefixExpression>(program);
                assert_eq!(expression.operator, self.operator);
                test_boolean_literal(expression.right.as_ref(), self.boolean_value);
            }
        }

        let tests: Vec<Box<dyn PrefixTest>> = vec![
            Box::new(IntegerPrefixTest {
                input: "!5".to_owned(),
                operator: "!".to_owned(),
                integer_value: 5,
            }),
            Box::new(IntegerPrefixTest {
                input: "-15".to_owned(),
                operator: "-".to_owned(),
                integer_value: 15,
            }),
            Box::new(BooleanPrefixTest {
                input: "!true".to_owned(),
                operator: "!".to_owned(),
                boolean_value: true,
            }),
            Box::new(BooleanPrefixTest {
                input: "!false".to_owned(),
                operator: "!".to_owned(),
                boolean_value: false,
            }),
        ];

        for test in tests {
            let lexer = Lexer::new(test.input());
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();
            assert_eq!(program.statements.len(), 1);

            test.test_expression(&program);
        }
    }

    fn test_integer_literal(expression: &dyn Expression, value: i64) {
        let integer_literal = expression
            .as_any()
            .downcast_ref::<IntegerLiteral>()
            .unwrap();
        assert_eq!(integer_literal.value, value);
    }

    fn test_identifier(expression: &dyn Expression, value: String) {
        let identifier = expression.as_any().downcast_ref::<Identifier>().unwrap();
        assert_eq!(identifier.value, value);
        assert_eq!(identifier.token_literal(), value);
    }

    fn test_boolean_literal(expression: &dyn Expression, value: bool) {
        let boolean_literal = expression.as_any().downcast_ref::<Boolean>().unwrap();
        assert_eq!(boolean_literal.value, value);
    }

    fn get_first_expression<T>(program: &Program) -> &T
    where
        T: 'static, // TODO: 去掉这个会有问题，好像是类型也需要一个生命周期（https://stackoverflow.com/questions/29740488/parameter-type-may-not-live-long-enough）
    {
        program
            .statements
            .first()
            .and_then(|statement| statement.as_any().downcast_ref::<ExpressionStatement>())
            .and_then(|expression_statement| {
                expression_statement.expression.as_any().downcast_ref::<T>()
            })
            .unwrap()
    }

    #[test]
    fn test_parsing_infix_expression() {
        trait InfixTest {
            fn input(&self) -> String;
            fn test_expression(&self, program: &Program);
        }

        // Go 里面直接把 left_value 的类型定义为了 interface {}，Rust 好像里面没有等价的表示任意类型的东西
        struct IntegerInfixTest {
            input: String,
            left_value: i64,
            operator: String,
            right_value: i64,
        }

        impl InfixTest for IntegerInfixTest {
            fn input(&self) -> String {
                self.input.clone()
            }

            fn test_expression(&self, program: &Program) {
                let expression = get_first_expression::<InfixExpression>(program);

                assert_eq!(expression.operator, self.operator);
                test_integer_literal(expression.left.as_ref(), self.left_value);

                test_integer_literal(expression.right.as_ref(), self.right_value);
            }
        }

        struct BooleanInfixTest {
            input: String,
            left_value: bool,
            operator: String,
            right_value: bool,
        }

        impl InfixTest for BooleanInfixTest {
            fn input(&self) -> String {
                self.input.clone()
            }

            fn test_expression(&self, program: &Program) {
                let expression = get_first_expression::<InfixExpression>(program);

                assert_eq!(expression.operator, self.operator);
                test_boolean_literal(expression.left.as_ref(), self.left_value);
                test_boolean_literal(expression.right.as_ref(), self.right_value);
            }
        }

        // 必须要用 Box 分配在 Heap 上
        let tests: Vec<Box<dyn InfixTest>> = vec![
            Box::new(IntegerInfixTest {
                input: "5 + 5;".to_owned(),
                left_value: 5,
                operator: "+".to_owned(),
                right_value: 5,
            }),
            Box::new(IntegerInfixTest {
                input: "5 - 5;".to_owned(),
                left_value: 5,
                operator: "-".to_owned(),
                right_value: 5,
            }),
            Box::new(IntegerInfixTest {
                input: "5 * 5;".to_owned(),
                left_value: 5,
                operator: "*".to_owned(),
                right_value: 5,
            }),
            Box::new(IntegerInfixTest {
                input: "5 / 5;".to_owned(),
                left_value: 5,
                operator: "/".to_owned(),
                right_value: 5,
            }),
            Box::new(IntegerInfixTest {
                input: "5 > 5;".to_owned(),
                left_value: 5,
                operator: ">".to_owned(),
                right_value: 5,
            }),
            Box::new(IntegerInfixTest {
                input: "5 < 5;".to_owned(),
                left_value: 5,
                operator: "<".to_owned(),
                right_value: 5,
            }),
            Box::new(IntegerInfixTest {
                input: "5 == 5;".to_owned(),
                left_value: 5,
                operator: "==".to_owned(),
                right_value: 5,
            }),
            Box::new(IntegerInfixTest {
                input: "5 != 5;".to_owned(),
                left_value: 5,
                operator: "!=".to_owned(),
                right_value: 5,
            }),
            Box::new(BooleanInfixTest {
                input: "true == true;".to_owned(),
                left_value: true,
                operator: "==".to_owned(),
                right_value: true,
            }),
            Box::new(BooleanInfixTest {
                input: "true != false;".to_owned(),
                left_value: true,
                operator: "!=".to_owned(),
                right_value: false,
            }),
            Box::new(BooleanInfixTest {
                input: "false == false;".to_owned(),
                left_value: false,
                operator: "==".to_owned(),
                right_value: false,
            }),
        ];

        for test in tests.iter() {
            let lexer = Lexer::new(test.input());
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();
            assert_eq!(program.statements.len(), 1);
            test.test_expression(&program);
        }
    }

    #[test]
    fn test_operator_precedence_parsing() {
        struct PrecedenceTest {
            input: String,
            expect: String,
        }

        let precedent_tests = [
            PrecedenceTest {
                input: "-a * b".to_owned(),
                expect: "((-a) * b)".to_owned(),
            },
            PrecedenceTest {
                input: "!-a".to_owned(),
                expect: "(!(-a))".to_owned(),
            },
            PrecedenceTest {
                input: "a + b + c".to_owned(),
                expect: "((a + b) + c)".to_owned(),
            },
            PrecedenceTest {
                input: "a + b - c".to_owned(),
                expect: "((a + b) - c)".to_owned(),
            },
            PrecedenceTest {
                input: "a * b * c".to_owned(),
                expect: "((a * b) * c)".to_owned(),
            },
            PrecedenceTest {
                input: "a * b / c".to_owned(),
                expect: "((a * b) / c)".to_owned(),
            },
            PrecedenceTest {
                input: "a + b / c".to_owned(),
                expect: "(a + (b / c))".to_owned(),
            },
            PrecedenceTest {
                input: "a + b * c + d / e - f".to_owned(),
                expect: "(((a + (b * c)) + (d / e)) - f)".to_owned(),
            },
            PrecedenceTest {
                input: "3 + 4; -5 * 5".to_owned(),
                expect: "(3 + 4)((-5) * 5)".to_owned(),
            },
            PrecedenceTest {
                input: "5 > 4 == 3 < 4".to_owned(),
                expect: "((5 > 4) == (3 < 4))".to_owned(),
            },
            PrecedenceTest {
                input: "5 < 4 != 3 > 4".to_owned(),
                expect: "((5 < 4) != (3 > 4))".to_owned(),
            },
            PrecedenceTest {
                input: "3 + 4 * 5 == 3 * 1 + 4 * 5".to_owned(),
                expect: "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))".to_owned(),
            },
            PrecedenceTest {
                input: "true".to_owned(),
                expect: "true".to_owned(),
            },
            PrecedenceTest {
                input: "false".to_owned(),
                expect: "false".to_owned(),
            },
            PrecedenceTest {
                input: "3 > 5 == false".to_owned(),
                expect: "((3 > 5) == false)".to_owned(),
            },
            PrecedenceTest {
                input: "3 < 5 == true".to_owned(),
                expect: "((3 < 5) == true)".to_owned(),
            },
            PrecedenceTest {
                input: "1 + (2 + 3) + 4".to_owned(),
                expect: "((1 + (2 + 3)) + 4)".to_owned(),
            },
            PrecedenceTest {
                input: "(5 + 5) * 2".to_owned(),
                expect: "((5 + 5) * 2)".to_owned(),
            },
            PrecedenceTest {
                input: "2 / (5 + 5)".to_owned(),
                expect: "(2 / (5 + 5))".to_owned(),
            },
            PrecedenceTest {
                input: "-(5 + 5)".to_owned(),
                expect: "(-(5 + 5))".to_owned(),
            },
            PrecedenceTest {
                input: "!(true == true)".to_owned(),
                expect: "(!(true == true))".to_owned(),
            },
            PrecedenceTest {
                input: "a + add(b * c) + d".to_owned(),
                expect: "((a + add((b * c))) + d)".to_owned(),
            },
            PrecedenceTest {
                input: "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))".to_owned(),
                expect: "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))".to_owned(),
            },
            PrecedenceTest {
                input: "add(a + b + c * d / f + g)".to_owned(),
                expect: "add((((a + b) + ((c * d) / f)) + g))".to_owned(),
            },
        ];

        for test in precedent_tests {
            let lexer = Lexer::new(test.input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();
            assert_eq!(program.string(), test.expect);
        }
    }

    #[test]
    fn test_boolean_expression() {
        let tests = [("true".to_owned(), true), ("false".to_owned(), false)];

        for test in tests {
            let lexer = Lexer::new(test.0);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();
            assert_eq!(program.statements.len(), 1);

            let bool_expression = get_first_expression::<Boolean>(&program);
            assert_eq!(bool_expression.value, test.1);
        }
    }

    #[test]
    fn test_if_expression() {
        let input = "if (x < y) { x }".to_owned();
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert_eq!(program.statements.len(), 1);

        let if_expression = get_first_expression::<IfExpression>(&program);
        let condition = if_expression
            .condition
            .as_any()
            .downcast_ref::<InfixExpression>()
            .unwrap();
        assert_eq!(condition.left.string(), "x");
        assert_eq!(condition.operator, "<");
        assert_eq!(condition.right.string(), "y");

        let consequence = if_expression
            .consequence
            .statements
            .first()
            .and_then(|statement| statement.as_any().downcast_ref::<ExpressionStatement>())
            .unwrap();
        test_identifier(consequence.expression.as_ref(), "x".to_owned());
        assert!(if_expression.alternative.is_none());
    }

    #[test]
    fn test_if_else_expression() {
        let input = "if (x < y) { x } else { y }".to_owned();
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert_eq!(program.statements.len(), 1);

        let if_expression = get_first_expression::<IfExpression>(&program);
        let condition = if_expression
            .condition
            .as_any()
            .downcast_ref::<InfixExpression>()
            .unwrap();
        assert_eq!(condition.left.string(), "x");
        assert_eq!(condition.operator, "<");
        assert_eq!(condition.right.string(), "y");

        let consequence = if_expression
            .consequence
            .statements
            .first()
            .and_then(|statement| statement.as_any().downcast_ref::<ExpressionStatement>())
            .unwrap();
        test_identifier(consequence.expression.as_ref(), "x".to_owned());
        let alternative = if_expression
            .alternative
            .as_ref()
            .and_then(|alt| alt.statements.first())
            .and_then(|statement| statement.as_any().downcast_ref::<ExpressionStatement>())
            .unwrap();
        test_identifier(alternative.expression.as_ref(), "y".to_owned());
    }

    #[test]
    fn test_function_literal() {
        let input = "fn(x, y) { x + y; }".to_owned();
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        for error in parser.error_messages {
            eprintln!("{}", error);
        }
        assert_eq!(program.statements.len(), 1);

        let function_literal = get_first_expression::<FunctionLiteral>(&program);
        assert_eq!(function_literal.parameters.len(), 2);
        test_identifier(&function_literal.parameters[0], "x".to_owned());
        test_identifier(&function_literal.parameters[1], "y".to_owned());

        assert_eq!(function_literal.body.statements.len(), 1);
        let expression = function_literal.body.statements[0]
            .as_any()
            .downcast_ref::<ExpressionStatement>()
            .and_then(|expression_statement| {
                expression_statement
                    .expression
                    .as_any()
                    .downcast_ref::<InfixExpression>()
            })
            .unwrap();
        assert_eq!(expression.left.string(), "x");
        assert_eq!(expression.operator, "+");
        assert_eq!(expression.right.string(), "y");
    }

    #[test]
    fn test_function_parameter_parsing() {
        struct ParameterTest {
            input: String,
            expected_params: Vec<String>,
        }

        let tests = [
            ParameterTest {
                input: "fn() {}".to_owned(),
                expected_params: Vec::new(),
            },
            ParameterTest {
                input: "fn(x) {}".to_owned(),
                expected_params: vec!["x".to_owned()],
            },
            ParameterTest {
                input: "fn(x, y, z) {}".to_owned(),
                expected_params: vec!["x".to_owned(), "y".to_owned(), "z".to_owned()],
            },
        ];

        for test in tests {
            let lexer = Lexer::new(test.input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();

            let function_literal = get_first_expression::<FunctionLiteral>(&program);
            assert_eq!(
                function_literal.parameters.len(),
                test.expected_params.len()
            );
            for (index, param) in test.expected_params.into_iter().enumerate() {
                test_identifier(&function_literal.parameters[index], param);
            }
        }
    }

    #[test]
    fn test_call_expression_parsing() {
        let input = "add(1, 2 * 3, 4 + 5);".to_owned();

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        let call_expression = get_first_expression::<CallExpression>(&program);

        test_identifier(call_expression.function.as_ref(), "add".to_owned());
        assert_eq!(call_expression.arguments.len(), 3);
        test_integer_literal(call_expression.arguments[0].as_ref(), 1);

        let second = call_expression.arguments[1]
            .as_any()
            .downcast_ref::<InfixExpression>()
            .unwrap();
        test_integer_literal(second.left.as_ref(), 2);
        assert_eq!(second.operator, "*");
        test_integer_literal(second.right.as_ref(), 3);

        let third = call_expression.arguments[2]
            .as_any()
            .downcast_ref::<InfixExpression>()
            .unwrap();
        test_integer_literal(third.left.as_ref(), 4);
        assert_eq!(third.operator, "+");
        test_integer_literal(third.right.as_ref(), 5);
    }
}
