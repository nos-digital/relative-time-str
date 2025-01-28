use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Value(Value),
    Operator(Operator),
    Unit(Unit),
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

pub fn next_token(s: &mut &str) -> Result<Option<Token>> {
    // Ignore any whitespace
    *s = s.trim_start();

    if s.is_empty() {
        return Ok(None);
    }

    let Some(c) = s.chars().next() else {
        return Ok(None);
    };

    let mut token = match c {
        '0'..='9' => Token::Value(Value::Number(0)),
        'n' => Token::Value(Value::Now),
        '/' => Token::Operator(Operator::Floor),
        '+' => Token::Operator(Operator::Add),
        '-' => Token::Operator(Operator::Sub),
        'y' => Token::Unit(Unit::Year),
        'M' => Token::Unit(Unit::Month),
        'w' => Token::Unit(Unit::Week),
        'd' => Token::Unit(Unit::Day),
        'h' => Token::Unit(Unit::Hour),
        'm' => Token::Unit(Unit::Minute),
        c => return Err(Error::InvalidCharacter(c)),
    };

    match &mut token {
        Token::Value(Value::Number(number)) => {
            let index = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
            let (digits, rest) = s.split_at(index);
            *number = digits.parse().expect("checked in find");
            *s = rest;
        }
        Token::Value(Value::Now) => {
            *s = &s[1..];
            *s = s.strip_prefix("ow").unwrap_or(s);
        }
        _ => {
            *s = &s[1..];
        }
    }

    Ok(Some(token))
}

pub fn next_step(s: &mut &str, first: &mut bool) -> Result<Option<Step>> {
    let is_first = *first;
    *first = false;
    let mut operator = if is_first { Some(Operator::Add) } else { None };
    let mut value = None;

    loop {
        match next_token(s)? {
            Some(Token::Value(v)) => {
                if operator.is_none() || operator == Some(Operator::Floor) {
                    return Err(Error::InvalidFormat(TokenType::Operator, TokenType::Value));
                }
                if operator == Some(Operator::Floor) || value.is_some() {
                    return Err(Error::InvalidFormat(TokenType::Unit, TokenType::Value));
                }
                value = Some(v);
            }
            Some(Token::Operator(o)) => {
                if operator.is_some() && !(operator == Some(Operator::Add) && o == Operator::Sub) {
                    if operator == Some(Operator::Floor) {
                        return Err(Error::InvalidFormat(TokenType::Unit, TokenType::Operator));
                    } else {
                        return Err(Error::InvalidFormat(TokenType::Value, TokenType::Operator));
                    }
                }
                if value.is_some() {
                    return Err(Error::InvalidFormat(TokenType::Unit, TokenType::Operator));
                }
                operator = Some(o);
            }
            Some(Token::Unit(unit)) => {
                let operator =
                    operator.ok_or(Error::InvalidFormat(TokenType::Operator, TokenType::Unit))?;

                return Ok(Some(match operator {
                    Operator::Add => {
                        let value =
                            value.ok_or(Error::InvalidFormat(TokenType::Value, TokenType::Unit))?;

                        Step::Add(value, unit)
                    }
                    Operator::Sub => {
                        let value =
                            value.ok_or(Error::InvalidFormat(TokenType::Value, TokenType::Unit))?;

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
                    return Err(Error::InvalidFormat(TokenType::Value, TokenType::None));
                }
                return Ok(None);
            }
        }
    }
}
