use crate::{parsing::TokenType, time_components::TimeComponents};

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error, Clone, Eq, PartialEq)]
pub enum Error {
    #[error("unexpected character '{1}' at position {0}")]
    UnexpectedCharacter(usize, char),
    #[error("number {0} is not valid: {1}")]
    InvalidNumber(String, std::num::ParseIntError),
    #[error("the computed date value is invalid")]
    InvalidTimestamp(TimeComponents),
    #[error("unexpected token at position {0}: expected {1}, found {2}")]
    InvalidFormat(usize, TokenType, TokenType),
}
