use crate::{lexer::Token, parser::Expression};

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error, Clone, Eq, PartialEq)]
pub enum Error {
    #[error("unexpected character '{1}' at position {0}")]
    UnexpectedCharacter(usize, char),
    #[error("number {0} is not valid: {1}")]
    InvalidNumber(String, std::num::ParseIntError),
    #[error("unexpected token at position {0}: expected {1}, found {2}")]
    InvalidFormat(usize, TokenType, TokenType),
    #[error("floor operation may not be done before 'now'")]
    FloorBeforeNow,
    #[error("'now' should occur once")]
    MissingNow,
    #[error("'now' cannot occur more than once")]
    MultipleNow,
    #[error("expression '{0}' is unsupported")]
    UnsupportedExpression(Expression),
    #[error("the given time delta is invalid")]
    InvalidDelta,
    #[error("the computed date value is invalid")]
    InvalidTimestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    // all individual tokens
    Now,
    Value,
    Add,
    Sub,
    Floor,
    Year,
    Month,
    Week,
    Day,
    Hour,
    Minute,
    Second,
    // additional tokentypes
    None,
    Operator,
    Unit,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Now => write!(f, "now"),
            Self::Value => write!(f, "number"),
            Self::Add => write!(f, "add"),
            Self::Sub => write!(f, "subtract"),
            Self::Floor => write!(f, "floor"),
            Self::Year => write!(f, "year"),
            Self::Month => write!(f, "month"),
            Self::Week => write!(f, "week"),
            Self::Day => write!(f, "day"),
            Self::Hour => write!(f, "hour"),
            Self::Minute => write!(f, "minute"),
            Self::Second => write!(f, "second"),
            Self::None => write!(f, "nothing"),
            Self::Operator => write!(f, "operator"),
            Self::Unit => write!(f, "unit"),
        }
    }
}

impl From<Token> for TokenType {
    fn from(value: Token) -> Self {
        match value {
            Token::Now => Self::Now,
            Token::Value(_) => Self::Value,
            Token::Add => Self::Add,
            Token::Sub => Self::Sub,
            Token::Floor => Self::Floor,
            Token::Year => Self::Year,
            Token::Month => Self::Month,
            Token::Week => Self::Week,
            Token::Day => Self::Day,
            Token::Hour => Self::Hour,
            Token::Minute => Self::Minute,
            Token::Second => Self::Second,
        }
    }
}
