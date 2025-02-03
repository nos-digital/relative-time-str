use crate::{parsing::TokenType, time_components::TimeComponents};

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
    #[error("character {1} is not valid syntax at character {0}")]
    InvalidCharacter(usize, char),
    #[error("number {0} is not valid: {1}")]
    InvalidNumber(String, std::num::ParseIntError),
    #[error("the computed date value is invalid")]
    InvalidTimestamp(TimeComponents),
    #[error("expected {1}, but got {2}, at character {0}")]
    InvalidFormat(usize, TokenType, TokenType),
}
