#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Keyword {
    Let,
    Lambda
}

#[derive(Debug, PartialEq, Eq)]
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
