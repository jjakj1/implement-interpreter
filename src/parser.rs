use by_address::ByAddress;
use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::ast::expressions::{
    ArrayLiteral, Boolean, CallExpression, FunctionLiteral, HashLiteral, Identifier, IfExpression,
    IndexExpression, InfixExpression, IntegerLiteral, MacroLiteral, PrefixExpression,
    StringLiteral,
};
use crate::ast::program::Program;
use crate::ast::statements::{BlockStatement, ExpressionStatement, LetStatement, ReturnStatement};
use crate::ast::traits::{Expression, Statement};
use crate::token::TokenType;
use crate::{lexer::Lexer, token::Token};

type PrefixParseFn = fn(&mut Parser) -> Result<Box<dyn Expression>, String>;
type InfixParseFn = fn(&mut Parser, Box<dyn Expression>) -> Result<Box<dyn Expression>, String>;

pub struct Parser {
    lexer: Lexer,
    current_token: Option<Token>,
    peek_token: Option<Token>,
    pub error_messages: Vec<String>,
    prefix_parse_fns: HashMap<TokenType, PrefixParseFn>,
    infix_parse_fns: HashMap<TokenType, InfixParseFn>,
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
    Index = 8,
}

static PRECEDENCES: Lazy<HashMap<TokenType, ExpressionPrecedence>> = Lazy::new(|| {
    HashMap::from([
        (TokenType::Equal, ExpressionPrecedence::Equals),
        (TokenType::NotEqual, ExpressionPrecedence::Equals),
        (TokenType::LessThan, ExpressionPrecedence::LessGreater),
        (TokenType::GreaterThan, ExpressionPrecedence::LessGreater),
        (TokenType::Plus, ExpressionPrecedence::Sum),
        (TokenType::Minus, ExpressionPrecedence::Sum),
        (TokenType::Slash, ExpressionPrecedence::Product),
        (TokenType::Asterisk, ExpressionPrecedence::Product),
        (TokenType::LeftParen, ExpressionPrecedence::Call),
        (TokenType::LeftBracket, ExpressionPrecedence::Index),
    ])
});

type HashLiteralPairsType = HashMap<ByAddress<Box<dyn Expression>>, Box<dyn Expression>>;

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
        parser.register_prefix(TokenType::Ident, Parser::parse_identifier);
        parser.register_prefix(TokenType::Int, Parser::parse_integer_literal);
        parser.register_prefix(TokenType::Bang, Parser::parse_prefix_expression);
        parser.register_prefix(TokenType::Minus, Parser::parse_prefix_expression);
        parser.register_prefix(TokenType::True, Parser::parse_boolean);
        parser.register_prefix(TokenType::False, Parser::parse_boolean);
        parser.register_prefix(TokenType::LeftParen, Parser::parse_grouped_expression);
        parser.register_prefix(TokenType::If, Parser::parse_if_expression);
        parser.register_prefix(TokenType::Function, Parser::parse_function_literal);
        parser.register_prefix(TokenType::String, Parser::parse_string_literal);
        parser.register_prefix(TokenType::LeftBracket, Parser::parse_array_literal);
        parser.register_prefix(TokenType::LeftBrace, Parser::parse_hash_literal);
        parser.register_prefix(TokenType::Macro, Parser::parse_macro_literal);

        parser.register_infix(TokenType::Plus, Parser::parse_infix_expression);
        parser.register_infix(TokenType::Minus, Parser::parse_infix_expression);
        parser.register_infix(TokenType::Slash, Parser::parse_infix_expression);
        parser.register_infix(TokenType::Asterisk, Parser::parse_infix_expression);
        parser.register_infix(TokenType::Equal, Parser::parse_infix_expression);
        parser.register_infix(TokenType::NotEqual, Parser::parse_infix_expression);
        parser.register_infix(TokenType::LessThan, Parser::parse_infix_expression);
        parser.register_infix(TokenType::GreaterThan, Parser::parse_infix_expression);
        parser.register_infix(TokenType::LeftParen, Parser::parse_call_expression);
        parser.register_infix(TokenType::LeftBracket, Parser::parse_index_expression);
        parser.next_token();
        parser.next_token();
        parser
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.take();
        self.peek_token = Some(self.lexer.next_token());
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program { statements: vec![] };

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
        let mut left_expression = prefix_parse_function(self)?;

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
                    left_expression = infix_parse_fn(self, left_expression)?;
                    self.next_token();
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
        let arguments = self.parse_expression_list(TokenType::RightParen)?;
        Ok(Box::new(CallExpression {
            token,
            function: left,
            arguments,
        }))
    }

    fn parse_expression_list(
        &mut self,
        end: TokenType,
    ) -> Result<Vec<Box<dyn Expression>>, String> {
        let mut args = Vec::new();
        self.next_token();
        if self.current_token_is(end) {
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

        self.expect_peek_token(end)?;
        Ok(args)
    }

    fn parse_index_expression(
        &mut self,
        left: Box<dyn Expression>,
    ) -> Result<Box<dyn Expression>, String> {
        let token = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .clone();
        self.next_token();
        let index = self.parse_expression(ExpressionPrecedence::Lowest)?;
        self.expect_peek_token(TokenType::RightBracket)?;
        Ok(Box::new(IndexExpression { token, left, index }) as Box<dyn Expression>)
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

    fn parse_string_literal(&mut self) -> Result<Box<dyn Expression>, String> {
        let token = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .clone();
        Ok(Box::new(StringLiteral {
            token: token.clone(),
            value: token.literal,
        }) as Box<dyn Expression>)
    }

    fn parse_array_literal(&mut self) -> Result<Box<dyn Expression>, String> {
        let token = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .clone();
        let elements = self.parse_expression_list(TokenType::RightBracket)?;
        Ok(Box::new(ArrayLiteral { token, elements }) as Box<dyn Expression>)
    }

    fn parse_hash_literal(&mut self) -> Result<Box<dyn Expression>, String> {
        let token = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .clone();
        let pairs = self.parse_expression_pair()?;
        Ok(Box::new(HashLiteral { token, pairs }) as Box<dyn Expression>)
    }

    fn parse_expression_pair(&mut self) -> Result<HashLiteralPairsType, String> {
        let mut pairs = HashMap::new();
        self.next_token();
        if self.current_token_is(TokenType::RightBrace) {
            return Ok(pairs);
        }

        loop {
            let key = self.parse_expression(ExpressionPrecedence::Lowest)?;
            self.expect_peek_token(TokenType::Colon)?;
            self.next_token();
            let value = self.parse_expression(ExpressionPrecedence::Lowest)?;
            pairs.insert(ByAddress(key), value);
            if self.peek_token_is(TokenType::Comma) {
                self.next_token();
                self.next_token();
            } else {
                break;
            }
        }

        self.expect_peek_token(TokenType::RightBrace)?;
        Ok(pairs)
    }

    fn parse_macro_literal(&mut self) -> Result<Box<dyn Expression>, String> {
        let token = self
            .current_token
            .as_ref()
            .ok_or("Current token is None")?
            .clone();
        self.expect_peek_token(TokenType::LeftParen)?;
        let parameters = self.parse_function_parameters()?;
        self.expect_peek_token(TokenType::LeftBrace)?;
        Ok(Box::new(MacroLiteral {
            token,
            parameters,
            body: self.parse_block_statement()?,
        }))
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

    fn register_prefix(&mut self, token_type: TokenType, fn_ptr: PrefixParseFn) {
        self.prefix_parse_fns.insert(token_type, fn_ptr);
    }

    fn register_infix(&mut self, token_type: TokenType, fn_ptr: InfixParseFn) {
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
