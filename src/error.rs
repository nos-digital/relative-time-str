use crate::{parsing::TokenType, time_components::TimeComponents};

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
    #[error("character {0} is not valid syntax")]
    InvalidCharacter(char),
    #[error("the computed date value is invalid")]
    InvalidTimestamp(TimeComponents),
    #[error("expected {0}, but got {1}")]
    InvalidFormat(TokenType, TokenType),
}
