use std::{iter::Peekable, str::CharIndices};

use crate::{Error, Result};

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

pub fn next_token(full: &str, s: &mut Peekable<CharIndices<'_>>) -> Result<Option<Token>> {
    let token = loop {
        let Some((index, c)) = s.next() else {
            return Ok(None);
        };

        break match c {
            '0'..='9' => {
                let mut index_end = index + 1;
                while matches!(s.peek(), Some((_, '0'..='9'))) {
                    index_end += 1;
                    s.next();
                }
                let number = full[index..index_end]
                    .parse::<u32>()
                    .map_err(|err| Error::InvalidNumber(full[index..index_end].to_string(), err))?;
                Token::Value(index, Value::Number(number))
            }
            'n' => {
                if matches!(s.peek(), Some((_, 'o'))) {
                    let (index, _) = s.next().unwrap();
                    match s.next() {
                        Some((_, 'w')) => {}
                        Some((index, c)) => return Err(Error::InvalidCharacter(index, c)),
                        None => return Err(Error::InvalidCharacter(index + 1, '\u{3}')), // 3 is EOT
                    }
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
            c if c.is_whitespace() => continue,
            c => return Err(Error::InvalidCharacter(index, c)),
        };
    };

    Ok(Some(token))
}

pub fn next_step(
    full: &str,
    s: &mut Peekable<CharIndices<'_>>,
    first: &mut bool,
) -> Result<Option<Step>> {
    let is_first = *first;
    *first = false;
    let mut operator = if is_first { Some(Operator::Add) } else { None };
    let mut value = None;

    loop {
        match next_token(full, s)? {
            Some(Token::Value(index, v)) => {
                if operator.is_none() || operator == Some(Operator::Floor) {
                    return Err(Error::InvalidFormat(
                        index,
                        TokenType::Operator,
                        TokenType::Value,
                    ));
                }
                if operator == Some(Operator::Floor) || value.is_some() {
                    return Err(Error::InvalidFormat(
                        index,
                        TokenType::Unit,
                        TokenType::Value,
                    ));
                }
                value = Some(v);
            }
            Some(Token::Operator(index, o)) => {
                if operator.is_some() && !(operator == Some(Operator::Add) && o == Operator::Sub) {
                    if operator == Some(Operator::Floor) {
                        return Err(Error::InvalidFormat(
                            index,
                            TokenType::Unit,
                            TokenType::Operator,
                        ));
                    } else {
                        return Err(Error::InvalidFormat(
                            index,
                            TokenType::Value,
                            TokenType::Operator,
                        ));
                    }
                }
                if value.is_some() {
                    return Err(Error::InvalidFormat(
                        index,
                        TokenType::Unit,
                        TokenType::Operator,
                    ));
                }
                operator = Some(o);
            }
            Some(Token::Unit(index, unit)) => {
                let operator = operator.ok_or(Error::InvalidFormat(
                    index,
                    TokenType::Operator,
                    TokenType::Unit,
                ))?;

                return Ok(Some(match operator {
                    Operator::Add => {
                        let value = value.ok_or(Error::InvalidFormat(
                            index,
                            TokenType::Value,
                            TokenType::Unit,
                        ))?;

                        Step::Add(value, unit)
                    }
                    Operator::Sub => {
                        let value = value.ok_or(Error::InvalidFormat(
                            index,
                            TokenType::Value,
                            TokenType::Unit,
                        ))?;

                        Step::Sub(value, unit)
                    }
                    Operator::Floor => {
                        // value is already checked in the Token::Value branch
                        Step::Floor(unit)
                    }
                }));
            }
            None => {
                if operator.is_some() && !is_first {
                    return Err(Error::InvalidFormat(
                        full.len(),
                        TokenType::Value,
                        TokenType::None,
                    ));
                }
                return Ok(None);
            }
        }
    }
}
