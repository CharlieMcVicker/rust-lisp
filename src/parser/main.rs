use super::super::lexer::main::Lexer;
use super::super::lexer::tokens::Token;
use super::super::lexer::tokens::Keyword;
use super::expressions::{Expression, Expression::*, literals};

use either::*;
use std::boxed::Box;
use std::rc::Rc;

#[derive(Debug)]
pub enum SyntaxError {
    EOF,
    MissingQuote,
    MissingParen,
    UnexpectedParen,
    BadOperator,
    MalformedLet,
    MalformedLambda,
    BadArgumentName,
    UnexpectedToken(Token)
}

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
    pub fn is_finished(&self) -> bool {
        self.lexer.is_finished()
    }
    pub fn parse(&mut self, skip_quote: bool) -> Result<Either<Expression, Token>, SyntaxError> {
        self.next();

        return match &self.current {
            Some(Token::OpenPar) if skip_quote => self.parse_list_expr(skip_quote).map(Either::Left),
            Some(Token::OpenPar) => self.parse_sexpr(skip_quote).map(Either::Left),
            Some(Token::QuoteToken) => self.parse_list_expr(skip_quote).map(Either::Left),
            Some(Token::IdentifierToken(lexeme)) => Ok(Left(Expression::LookupExpr(lexeme.to_string()))),
            Some(Token::IntLiteral(lexeme)) => Ok(Left(literals::integer(lexeme.to_string()))),
            Some(Token::FloatLiteral(lexeme)) => Ok(Left(literals::float(lexeme.to_string()))),
            Some(Token::StringLiteral(lexeme)) => Ok(Left(literals::string(lexeme.to_string()))),
            Some(Token::ClosePar) => Ok(Right(Token::ClosePar)),
            Some(Token::QuoteToken) => Ok(Right(Token::QuoteToken)),
            Some(Token::Unknown(lexeme)) => Ok(Right(Token::Unknown(lexeme.to_string()))),
            Some(Token::KeywordToken(keyword)) => Ok(Right(Token::KeywordToken(*keyword))),
            Some(Token::EOF) => Ok(Right(Token::EOF)),
            None => Err(SyntaxError::EOF),
        };
    }
    fn parse_list_expr(&mut self, skip_quote: bool) -> Result<Expression, SyntaxError> {
        if !skip_quote {
            if self.current.as_ref().map_or(false, |c| *c != Token::QuoteToken) {
                return Err(SyntaxError::MissingQuote);
            }
        }
        if self.current.as_ref().map_or(false, |c| *c != Token::OpenPar) {
                return Err(SyntaxError::MissingParen);
        }
        let mut contents = Vec::new();
        let mut runner = self.parse(true)?;
        while runner.as_ref().either(|expr| true, |tok| *tok != Token::ClosePar) {
            match runner {
                Left(expr) => {
                    contents.push(Box::new(expr));
                    runner = self.parse(true)?;
                },
                Right(tok) => return Err(SyntaxError::UnexpectedToken(tok)),
            }
        }
        return Ok(Expression::ListExpr(contents));
    }
    fn parse_sexpr(&mut self, skip_quote: bool) -> Result<Expression, SyntaxError> {
        if self.current.as_ref().map_or(false, |c| *c != Token::OpenPar) {
            return Err(SyntaxError::MissingParen);
        }
        let func = self.parse(skip_quote)?;
        let mut args = Vec::new();
        let mut runner = self.parse(skip_quote)?;
        while runner.as_ref().either(|expr| true, |tok| *tok != Token::ClosePar) {
            match runner {
                Left(expr) => {
                    args.push(Box::new(expr));
                    runner = self.parse(skip_quote)?;
                },
                Right(tok) => return Err(SyntaxError::UnexpectedToken(tok)),
            }
        }
        match func {
            Right(Token::KeywordToken(keyword)) => match keyword {
                Keyword::Lambda => self.parse_lambda_expr(args),
                Keyword::Let => self.parse_let_expr(args),
            },
            Left(expr) => Ok(Expression::SExpr(Box::new(expr), args)),
            _ => Err(SyntaxError::BadOperator) 
        }
    }
    fn parse_arg_names(arg_list: Vec<Box<Expression>>) -> Result<Vec<String>, SyntaxError> {
        // collecting into a Result < Vec<String>, SyntaxError > causes us to fail out at the first
        // error, as desired
        let arg_names: Result<Vec<String>, SyntaxError> = arg_list.into_iter().map(|arg| match *arg {
            Expression::LookupExpr(arg_name) => Ok(arg_name),
            _ => Err(SyntaxError::BadArgumentName)
        }).collect();
        return arg_names;
    }
    fn parse_let_expr(&mut self, mut args: Vec<Box<Expression>>) -> Result<Expression, SyntaxError> {
        if args.len() != 2 {
            return Err(SyntaxError::MalformedLet);
        }
        let name = args.remove(0);
        let body = args.remove(0);
        match *name {
            Expression::LookupExpr(lexeme) => Ok(Expression::LetExpr(lexeme, body)),
            Expression::SExpr(rator, arg_list) => match *rator {
                Expression::LookupExpr(func_name) => {
                    let arg_names = Parser::parse_arg_names(arg_list)?;
                    Ok(Expression::LetExpr(func_name, Box::new(Expression::LambdaExpr(arg_names, Rc::new(*body)))))
                },
                _ => Err(SyntaxError::MalformedLet)
            },
            _ => Err(SyntaxError::MalformedLet)
        }
    }
    fn parse_lambda_expr(&mut self, mut args: Vec<Box<Expression>>) -> Result<Expression, SyntaxError> {
        if args.len() != 2 {
            return Err(SyntaxError::MalformedLambda)
        }
        let arg_list = args.remove(0);
        let body = args.remove(0);
        match *arg_list {
            SExpr(car, mut cdr) => {
                // TODO: PARSE LAMBDA DIFFERENTLY (w/ skip quote)
                let mut arg_list = vec![car];
                arg_list.append(&mut cdr);
                let arg_names = Parser::parse_arg_names(arg_list)?;
                Ok(LambdaExpr(arg_names, Rc::new(*body)))
            },
            _ => Err(SyntaxError::MalformedLambda)
        }
    }
}
