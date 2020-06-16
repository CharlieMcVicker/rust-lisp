# rust-lisp
This project is just for fun! It is a lisp intepreter written in three steps. The whole process of interpretation is treated as converting between three or four data types. There is a `lexer` module which converts Strings into `Token`s, a `parser` module which converts `Token`s into `Expression`s, and a runtime module which converts `Expression`s into `Value`s. Those data types are defined as:

```rust
// src/lexer/tokens.rs

pub enum Token {
    EOF,
    IdentifierToken(String),
    IntLiteral(String),
    FloatLiteral(String),
    StringLiteral(String),
    OpenPar,
    ClosePar,
    QuoteToken,
    Unknown(String),
    KeywordToken(Keyword)
}

// src/parser/expressions.rs

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

// src/runtime/main.rs

pub enum Value {
    Nil,
    Lambda(Rc<LambdaFunction>),
    RuntimeFunction(RuntimeFunctionWrapper),
    Int(i32),
    Float(f32),
    Str(String)
}

pub enum RuntimeFunctionWrapper {
    Immediate(fn (Rc<Env>, Vec<Rc<Value>>) -> (Rc<Env>, Rc<Value>)),
    Symbolic(fn (Rc<Env>, Vec<Box<Expression>>) -> (Rc<Env>, Rc<Value>))
}

struct LambdaFunction {
    env: Rc<Env>,
    arg_names: Vec<String>,
    body: Rc<Expression>,
    own_name: Option<String>
}

pub struct Env {
    table: HashMap<String, Rc<Value>>
}
```
