use std::rc::Rc;
use std::boxed::Box;

#[derive(Debug, Clone)]
pub enum Expression {
    ListExpr(Vec<Box<Expression>>),
    SExpr(Box<Expression>, Vec<Box<Expression>>),
    LetExpr(String, Box<Expression>),
    LambdaExpr(Vec<String>, Rc<Expression>),
    LookupExpr(String),
    IntegerLiteral(i32),
    FloatLiteral(f32),
    StringLiteral(String)
}

pub mod literals {
    use super::Expression;
    pub fn integer(lexeme: String) -> Expression {
        Expression::IntegerLiteral(lexeme.parse::<i32>().unwrap())
    }
    pub fn float(lexeme: String) -> Expression {
        Expression::FloatLiteral(lexeme.parse::<f32>().unwrap())
    }
    pub fn string(lexeme: String) -> Expression {
        Expression::StringLiteral(lexeme)
    }
}
