use std::{iter::Peekable, str::CharIndices};

use crate::{Error, Result};

macro_rules! bail {
    ($err:expr) => {
        return Some(Err($err))
    };
}

macro_rules! ensure_ok {
    ($value:expr) => {
        match $value {
            Ok(value) => value,
            Err(err) => return Some(Err(err)),
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Value(usize, Value),
    Operator(usize, Operator),
    Unit(usize, Unit),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Value,
    Operator,
    Unit,
    None,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value => write!(f, "number or now"),
            Self::Operator => write!(f, "operator"),
            Self::Unit => write!(f, "unit"),
            Self::None => write!(f, "nothing"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Now,
    Number(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Add,
    Sub,
    Floor,
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

pub enum Step {
    Add(Value, Unit),
    Sub(Value, Unit),
    Floor(Unit),
}

pub struct TokenIterator<'s> {
    text: &'s str,
    chars: Peekable<CharIndices<'s>>,
}

impl<'s> TokenIterator<'s> {
    pub fn new(text: &'s str) -> Self {
        Self {
            text,
            chars: text.char_indices().peekable(),
        }
    }
}

impl Iterator for TokenIterator<'_> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, c) = self.chars.next()?;
        Some(Ok(match c {
            '0'..='9' => {
                let mut index_end = index + 1;
                while matches!(self.chars.peek(), Some((_, '0'..='9'))) {
                    index_end += 1;
                    self.chars.next();
                }
                match self.text[index..index_end].parse::<u32>() {
                    Ok(number) => Token::Value(index, Value::Number(number)),
                    Err(err) => bail!(Error::InvalidNumber(
                        self.text[index..index_end].to_string(),
                        err
                    )),
                }
            }
            'n' => {
                match self.chars.next() {
                    Some((_, 'o')) => {}
                    Some((index, c)) => bail!(Error::InvalidCharacter(index, c)),
                    None => bail!(Error::InvalidCharacter(index + 1, '\u{3}')), // 3 is EOT
                }
                match self.chars.next() {
                    Some((_, 'w')) => {}
                    Some((index, c)) => bail!(Error::InvalidCharacter(index, c)),
                    None => bail!(Error::InvalidCharacter(index + 1, '\u{3}')), // 3 is EOT
                }
                Token::Value(index, Value::Now)
            }
            '/' => Token::Operator(index, Operator::Floor),
            '+' => Token::Operator(index, Operator::Add),
            '-' => Token::Operator(index, Operator::Sub),
            'y' => Token::Unit(index, Unit::Year),
            'M' => Token::Unit(index, Unit::Month),
            'w' => Token::Unit(index, Unit::Week),
            'd' => Token::Unit(index, Unit::Day),
            'h' => Token::Unit(index, Unit::Hour),
            'm' => Token::Unit(index, Unit::Minute),
            c if c.is_whitespace() => return self.next(),
            c => bail!(Error::InvalidCharacter(index, c)),
        }))
    }
}

pub struct StepIterator<'s> {
    first: bool,
    tokens: TokenIterator<'s>,
}

impl<'s> StepIterator<'s> {
    pub fn new(text: &'s str) -> Self {
        Self {
            first: true,
            tokens: TokenIterator::new(text),
        }
    }

    pub fn next_value(&mut self) -> Result<Value> {
        match self.tokens.next().transpose()? {
            Some(Token::Value(_, value)) => Ok(value),
            Some(Token::Operator(index, _)) => Err(Error::InvalidFormat(
                index,
                TokenType::Value,
                TokenType::Operator,
            )),
            Some(Token::Unit(index, _)) => Err(Error::InvalidFormat(
                index,
                TokenType::Value,
                TokenType::Unit,
            )),
            None => Err(Error::InvalidFormat(
                self.tokens.text.len(),
                TokenType::Value,
                TokenType::None,
            )),
        }
    }
    pub fn next_unit(&mut self) -> Result<Unit> {
        match self.tokens.next().transpose()? {
            Some(Token::Unit(_, unit)) => Ok(unit),
            Some(Token::Operator(index, _)) => Err(Error::InvalidFormat(
                index,
                TokenType::Unit,
                TokenType::Operator,
            )),
            Some(Token::Value(index, _)) => Err(Error::InvalidFormat(
                index,
                TokenType::Unit,
                TokenType::Value,
            )),
            None => Err(Error::InvalidFormat(
                self.tokens.text.len(),
                TokenType::Unit,
                TokenType::None,
            )),
        }
    }
}

impl Iterator for StepIterator<'_> {
    type Item = Result<Step>;

    fn next(&mut self) -> Option<Self::Item> {
        let is_first = self.first;
        self.first = false;

        Some(match ensure_ok!(self.tokens.next().transpose()) {
            Some(Token::Operator(index, Operator::Floor)) if is_first => Err(Error::InvalidFormat(
                index,
                TokenType::Operator,
                TokenType::Operator,
            )),
            Some(Token::Operator(_, operator)) => match operator {
                Operator::Add => {
                    let value = ensure_ok!(self.next_value());
                    let unit = ensure_ok!(self.next_unit());
                    Ok(Step::Add(value, unit))
                }
                Operator::Sub => {
                    let value = ensure_ok!(self.next_value());
                    let unit = ensure_ok!(self.next_unit());
                    Ok(Step::Sub(value, unit))
                }
                Operator::Floor => {
                    let unit = ensure_ok!(self.next_unit());
                    Ok(Step::Floor(unit))
                }
            },
            Some(Token::Value(_, value)) if is_first => {
                let unit = ensure_ok!(self.next_unit());
                Ok(Step::Add(value, unit))
            }
            Some(Token::Value(index, _)) => Err(Error::InvalidFormat(
                index,
                TokenType::Operator,
                TokenType::Value,
            )),
            Some(Token::Unit(index, _)) => Err(Error::InvalidFormat(
                index,
                TokenType::Operator,
                TokenType::Unit,
            )),
            None => Err(Error::InvalidFormat(
                self.tokens.text.len(),
                TokenType::Operator,
                TokenType::None,
            )),
        })
    }
}
