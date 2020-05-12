use super::super::lexer::main::Lexer;
use super::super::lexer::tokens::Token;
use super::super::lexer::tokens::Keyword;
use super::expressions::{Expression, Expression::*, literals};

use either::*;
use std::boxed::Box;
use std::rc::Rc;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Option<Token>,
    verbose: u8
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a String, verbose: u8) -> Self {
        let mut lexer = Lexer::new(source, verbose);
        Parser {
            lexer: lexer,
            current: None,
            verbose: verbose
        }
    }
    fn next(&mut self) {
        self.current = self.lexer.lex();
    }
    pub fn parse(&mut self, skip_quote: bool) -> Either<Expression, Token> {
        self.next();
        if skip_quote {
            match self.current {
                Some(Token::OpenPar) => return Left(self.parse_list_expr(skip_quote)),
                _ => {}
            }
        }

        return match &self.current {
            Some(Token::OpenPar) => Left(self.parse_sexpr(skip_quote)),
            Some(Token::QuoteToken) => Left(self.parse_list_expr(skip_quote)),
            Some(Token::IdentifierToken(lexeme)) => Left(Expression::LookupExpr(lexeme.to_string())),
            Some(Token::IntLiteral(lexeme)) => Left(literals::integer(lexeme.to_string())),
            Some(Token::FloatLiteral(lexeme)) => Left(literals::float(lexeme.to_string())),
            Some(Token::StringLiteral(lexeme)) => Left(literals::string(lexeme.to_string())),
            Some(Token::ClosePar) => Right(Token::ClosePar),
            Some(Token::QuoteToken) => Right(Token::QuoteToken),
            Some(Token::Unknown(lexeme)) => Right(Token::Unknown(lexeme.to_string())),
            Some(Token::KeywordToken(keyword)) => Right(Token::KeywordToken(*keyword)),
            Some(Token::EOF) => Right(Token::EOF),
            None => Right(Token::EOF),
        };
    }
    fn parse_list_expr(&mut self, skip_quote: bool) -> Expression {
        if !skip_quote {
            if self.current.as_ref().map_or(false, |c| c != &Token::QuoteToken) {
               panic!("Missing quote at start of list"); 
            }
        }
        if self.current.as_ref().map_or(false, |c| c != &Token::OpenPar) {
            panic!("Missing open paren at start of list");
        }
        let mut contents = Vec::new();
        let mut runner = self.parse(true);
        while runner.as_ref().either(|expr| true, |tok| tok != &Token::ClosePar) {
            contents.push(Box::new(runner.left().unwrap()));
            runner = self.parse(true);
        }
        return Expression::ListExpr(contents);
    }
    fn parse_sexpr(&mut self, skip_quote: bool) -> Expression {
        if self.current.as_ref().map_or(false, |c| *c != Token::OpenPar) {
            println!("{:?}", self.current);
            panic!("Missing open paren at start of sexpr");
        }
        let func = self.parse(skip_quote);
        let mut args = Vec::new();
        let mut runner = self.parse(skip_quote);
        while runner.as_ref().either(|expr| true, |tok| tok != &Token::ClosePar) {
            args.push(Box::new(runner.left().unwrap()));
            runner = self.parse(skip_quote);
        }
        match func {
            Right(Token::KeywordToken(keyword)) => match keyword {
                Let => self.parse_let_expr(args),
                Lambda => self.parse_lambda_expr(args)
            },
            Left(expr) => Expression::SExpr(Box::new(expr), args),
            _ => panic!("Bad function in s-expr")
        }
    }
    fn parse_let_expr(&mut self, mut args: Vec<Box<Expression>>) -> Expression {
        if args.len() != 2 {
            panic!("Too many arguments for let expression");
        }
        let name = args.remove(0);
        let body = args.remove(0);
        match *name {
            Expression::LookupExpr(lexeme) => Expression::LetExpr(lexeme, body),
            Expression::SExpr(rator, arg_list) => match *rator {
                Expression::LookupExpr(func_name) => {
                    let arg_names: Vec<String> = arg_list.into_iter().map(|arg| match *arg {
                        Expression::LookupExpr(arg_name) => arg_name,
                        _ => panic!("Invalid argument name")
                    }).collect();
                    Expression::LetExpr(func_name, Box::new(Expression::LambdaExpr(arg_names, Rc::new(*body))))
                },
                _ => panic!("Invalid function name")
            },
            _ => panic!("Invalid name for let expression")
        }
    }
    fn parse_lambda_expr(&mut self, mut args: Vec<Box<Expression>>) -> Expression {
        if args.len() != 2 {
            panic!("Too many arguments for lambda creating expression");
        }
        let arg_list = args.remove(0);
        let body = args.remove(0);
        match *arg_list {
            SExpr(car, cdr) => {
                let arg_names: Vec<String> = vec![*car].into_iter().chain(cdr.into_iter().map(|v| *v)).map(|arg| match arg {
                    LookupExpr(arg_name) => arg_name,
                    _ => panic!("Invalid argument name")
                }).collect();
                LambdaExpr(arg_names, Rc::new(*body))
            },
            _ => panic!("Malformed argument list")
        }
    }
}
