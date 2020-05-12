use super::tokens::{Token, Keyword};
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    source: &'a String,
    buffer: Peekable<Chars<'a>>,
    current: char,
    finished: bool,
    verbosity: u8 // TODO: make verbosity enum
}

const OPERATORS: [char; 5] = ['+', '-' , '*', '/', '='];

impl<'a> Lexer<'a> {
    pub fn new (src: &'a String, verbosity: u8) -> Self {
        let mut buff = src.chars().peekable();
        let current = *buff.peek().unwrap_or(&'\0');
        let mut l = Lexer {
            source: src,
            finished: false,
            buffer: buff,
            current: current,
            verbosity: verbosity,
        };
        l.next();
        return l;
    }
    pub fn is_finished(&self) -> bool {
        self.finished
    }
    fn next(&mut self) {
        match self.buffer.next() {
            Some(c) => {
                self.current = c;
            },
            None => {
                self.finished = true;
                self.current = '\0';
            }
        }
    }
    fn next_nw(&mut self) {
        while self.current.is_whitespace() {
            self.next();
        }
    }
    pub fn lex(&mut self) -> Option<Token> {
        self.next_nw();
        if self.current == '\0' {
            return Some(Token::EOF);
        } else if self.current == '"' {
            return self.lex_string();
        } else if self.current.is_alphabetic() {
            return self.lex_ident_or_kw();
        } else if self.current.is_digit(10) {
            return self.lex_number();
        } else {
            return match self.lex_special() {
                Some(tok) => Some(tok),
                None => {
                    Some(Token::Unknown(self.current.to_string()))
                }
            };
        }
    }
    fn lex_string(&mut self) -> Option<Token> {
        let mut lexeme = String::new();
        self.next();
        while self.current != '"' {
            if self.current == '\\' {
                self.next();
            }
            lexeme.push(self.current);
            self.next();
        }
        self.next(); // move off of breaking '"'
        Some(Token::StringLiteral(lexeme))
    }
    fn lex_ident_or_kw(&mut self) -> Option<Token> {
        let mut lexeme = String::new();
        while self.current.is_alphabetic() || self.current.is_digit(10) {
            lexeme.push(self.current);
            self.next();
        }
        if lexeme == "let" {
            Some(Token::KeywordToken(Keyword::Let))
        } else if lexeme == "lambda" {
            Some(Token::KeywordToken(Keyword::Lambda))
        } else {
            Some(Token::IdentifierToken(lexeme))
        }
    }
    fn lex_number(&mut self) -> Option<Token> {
        let mut lexeme = String::new();
        while self.current.is_digit(10) {
            lexeme.push(self.current);
            self.next()
        }
        if self.current == '.' {
            lexeme.push(self.current);
            self.next();
            while self.current.is_digit(10) {
                lexeme.push(self.current);
                self.next()
            }
            return Some(Token::FloatLiteral(lexeme));
        } else {
            Some(Token::IntLiteral(lexeme))
        }
    }
    fn lex_special(&mut self) -> Option<Token> {
        if self.current == '(' {
            self.next();
            Some(Token::OpenPar)
        } else if self.current == ')' {
            self.next();
            Some(Token::ClosePar)
        } else if self.current == '\'' {
            self.next();
            Some(Token::QuoteToken)
        } else if OPERATORS.iter().any(|v| v == &self.current) {
            let lexeme = self.current.to_string();
            self.next();
            Some(Token::IdentifierToken(lexeme))
        } else {
            None
        }
    }
}
