use std::iter::Peekable;

use crate::{
    error::TokenType,
    lexer::{Lexer, Token},
    Error, Result,
};

macro_rules! ensure_ok {
    ($value:expr) => {
        match $value {
            Some(Ok(value)) => Some(value),
            Some(Err(err)) => return Some(Err(err)),
            None => None,
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Year,
    Month,
    Week,
    Day,
    Hour,
    Minute,
    Second,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operator {
    Add,
    Sub,
    Floor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expression {
    Now,
    Add(u32, Unit),
    Sub(u32, Unit),
    Floor(Unit),
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Now => write!(f, "now"),
            Self::Add(_, unit) => write!(f, "add {}", unit),
            Self::Sub(_, unit) => write!(f, "subtract {}", unit),
            Self::Floor(unit) => write!(f, "floor {}", unit),
        }
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Year => write!(f, "year"),
            Self::Month => write!(f, "month"),
            Self::Week => write!(f, "week"),
            Self::Day => write!(f, "day"),
            Self::Hour => write!(f, "hour"),
            Self::Minute => write!(f, "minute"),
            Self::Second => write!(f, "second"),
        }
    }
}

pub struct Parser<'s> {
    first: bool,
    tokens: Peekable<Lexer<'s>>,
}

impl<'s> Parser<'s> {
    pub fn new(text: &'s str) -> Self {
        Self {
            first: true,
            tokens: Lexer::new(text).peekable(),
        }
    }

    fn next_expression(&mut self) -> Option<Result<Expression>> {
        Some(Ok(match ensure_ok!(self.next_operator())? {
            Operator::Add => {
                if matches!(self.tokens.peek(), Some(Ok((_, Token::Now)))) {
                    self.tokens.next(); // discard peeked
                    Expression::Now
                } else {
                    let value = ensure_ok!(self.next_value())?;
                    let unit = ensure_ok!(self.next_unit())?;
                    Expression::Add(value, unit)
                }
            }
            Operator::Sub => {
                let value = ensure_ok!(self.next_value())?;
                let unit = ensure_ok!(self.next_unit())?;
                Expression::Sub(value, unit)
            }
            Operator::Floor => {
                let unit = ensure_ok!(self.next_unit())?;
                Expression::Floor(unit)
            }
        }))
    }
    fn next_operator(&mut self) -> Option<Result<Operator>> {
        // allow omitting an initial +
        if self.first {
            self.first = false;
            if matches!(
                self.tokens.peek(),
                Some(Ok((_, Token::Now | Token::Value(_))))
            ) {
                return Some(Ok(Operator::Add));
            }
        }
        match ensure_ok!(self.tokens.next())? {
            (_index, Token::Add) => Some(Ok(Operator::Add)),
            (_index, Token::Sub) => Some(Ok(Operator::Sub)),
            (_index, Token::Floor) => Some(Ok(Operator::Floor)),
            (index, token) => Some(Err(Error::InvalidFormat(
                index,
                TokenType::Operator,
                token.into(),
            ))),
        }
    }
    fn next_value(&mut self) -> Option<Result<u32>> {
        Some(match ensure_ok!(self.tokens.next())? {
            (_index, Token::Value(value)) => Ok(value),
            (index, token) => Err(Error::InvalidFormat(index, TokenType::Value, token.into())),
        })
    }
    fn next_unit(&mut self) -> Option<Result<Unit>> {
        Some(match ensure_ok!(self.tokens.next())? {
            (_index, Token::Year) => Ok(Unit::Year),
            (_index, Token::Month) => Ok(Unit::Month),
            (_index, Token::Week) => Ok(Unit::Week),
            (_index, Token::Day) => Ok(Unit::Day),
            (_index, Token::Hour) => Ok(Unit::Hour),
            (_index, Token::Minute) => Ok(Unit::Minute),
            (_index, Token::Second) => Ok(Unit::Second),
            (index, token) => Err(Error::InvalidFormat(index, TokenType::Unit, token.into())),
        })
    }
}

impl Iterator for Parser<'_> {
    type Item = Result<Expression>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_expression()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn parse(input: &str) -> Result<Vec<Expression>, Error> {
        Parser::new(input).collect::<Result<Vec<_>, _>>()
    }

    macro_rules! parse_eq {
        ($string:expr, $vector:expr) => {
            assert_eq!(parse($string), $vector);
        };
    }
    macro_rules! parse_matches {
        ($string:expr, $pattern:pat) => {
            assert!(matches!(parse($string), $pattern));
        };
    }

    #[test]
    fn now() {
        parse_eq!("now", Ok(vec![Expression::Now]));
    }

    #[test]
    fn add_now() {
        parse_eq!("+now", Ok(vec![Expression::Now]));
    }

    #[test]
    fn now_add_year() {
        parse_eq!(
            "now+1y",
            Ok(vec![Expression::Now, Expression::Add(1, Unit::Year)])
        );
    }

    #[test]
    fn add_seconds_now() {
        parse_eq!(
            "+1s+now",
            Ok(vec![Expression::Add(1, Unit::Second), Expression::Now])
        );
    }

    #[test]
    fn sub_day_now() {
        parse_eq!(
            "-5d+now",
            Ok(vec![Expression::Sub(5, Unit::Day), Expression::Now])
        );
    }

    #[test]
    fn floor_week() {
        parse_eq!(
            "now/w",
            Ok(vec![Expression::Now, Expression::Floor(Unit::Week)])
        );
    }

    #[test]
    fn add_zero() {
        parse_eq!(
            "now+0y-0m+0s",
            Ok(vec![
                Expression::Now,
                Expression::Add(0, Unit::Year),
                Expression::Sub(0, Unit::Minute),
                Expression::Add(0, Unit::Second)
            ])
        );
    }

    #[test]
    fn day_sub_now() {
        parse_matches!("1d-now", Err(Error::InvalidFormat(3, _, _)));
    }

    #[test]
    fn no_input() {
        parse_eq!("", Ok(vec![]));
    }
}
