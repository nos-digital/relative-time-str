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
        loop {
            let (index, c) = self.chars.next()?;

            return Some(match c {
                '0'..='9' => {
                    let mut index_end = index + 1;
                    while matches!(self.chars.peek(), Some((_, '0'..='9'))) {
                        index_end += 1;
                        self.chars.next();
                    }
                    self.text[index..index_end]
                        .parse::<u32>()
                        .map(|number| Token::Value(index, Value::Number(number)))
                        .map_err(|err| {
                            Error::InvalidNumber(self.text[index..index_end].to_string(), err)
                        })
                }
                'n' => {
                    match self.chars.next() {
                        Some((_, 'o')) => {}
                        Some((index, c)) => return Some(Err(Error::InvalidCharacter(index, c))),
                        None => return Some(Err(Error::InvalidCharacter(index + 1, '\u{3}'))), // 3 is EOT
                    }
                    match self.chars.next() {
                        Some((_, 'w')) => {}
                        Some((index, c)) => return Some(Err(Error::InvalidCharacter(index, c))),
                        None => return Some(Err(Error::InvalidCharacter(index + 1, '\u{3}'))), // 3 is EOT
                    }
                    Ok(Token::Value(index, Value::Now))
                }
                '/' => Ok(Token::Operator(index, Operator::Floor)),
                '+' => Ok(Token::Operator(index, Operator::Add)),
                '-' => Ok(Token::Operator(index, Operator::Sub)),
                'y' => Ok(Token::Unit(index, Unit::Year)),
                'M' => Ok(Token::Unit(index, Unit::Month)),
                'w' => Ok(Token::Unit(index, Unit::Week)),
                'd' => Ok(Token::Unit(index, Unit::Day)),
                'h' => Ok(Token::Unit(index, Unit::Hour)),
                'm' => Ok(Token::Unit(index, Unit::Minute)),
                c if c.is_whitespace() => continue,
                c => return Some(Err(Error::InvalidCharacter(index, c))),
            });
        }
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
}

impl Iterator for StepIterator<'_> {
    type Item = Result<Step>;

    fn next(&mut self) -> Option<Self::Item> {
        let was_first = self.first;
        self.first = false;

        let mut operator = if was_first { Some(Operator::Add) } else { None };
        let mut value = None;

        loop {
            match self.tokens.next() {
                Some(Err(err)) => {
                    return Some(Err(err));
                }
                Some(Ok(Token::Value(index, v))) => {
                    if operator.is_none() || operator == Some(Operator::Floor) {
                        return Some(Err(Error::InvalidFormat(
                            index,
                            TokenType::Operator,
                            TokenType::Value,
                        )));
                    }
                    if operator == Some(Operator::Floor) || value.is_some() {
                        return Some(Err(Error::InvalidFormat(
                            index,
                            TokenType::Unit,
                            TokenType::Value,
                        )));
                    }
                    value = Some(v);
                }
                Some(Ok(Token::Operator(index, o))) => {
                    if operator.is_some()
                        && !(operator == Some(Operator::Add) && o == Operator::Sub)
                    {
                        if operator == Some(Operator::Floor) {
                            return Some(Err(Error::InvalidFormat(
                                index,
                                TokenType::Unit,
                                TokenType::Operator,
                            )));
                        } else {
                            return Some(Err(Error::InvalidFormat(
                                index,
                                TokenType::Value,
                                TokenType::Operator,
                            )));
                        }
                    }
                    if value.is_some() {
                        return Some(Err(Error::InvalidFormat(
                            index,
                            TokenType::Unit,
                            TokenType::Operator,
                        )));
                    }
                    operator = Some(o);
                }
                Some(Ok(Token::Unit(index, unit))) => {
                    let Some(operator) = operator else {
                        return Some(Err(Error::InvalidFormat(
                            index,
                            TokenType::Operator,
                            TokenType::Unit,
                        )));
                    };

                    return Some(Ok(match operator {
                        Operator::Add => {
                            let Some(value) = value else {
                                return Some(Err(Error::InvalidFormat(
                                    index,
                                    TokenType::Value,
                                    TokenType::Unit,
                                )));
                            };

                            Step::Add(value, unit)
                        }
                        Operator::Sub => {
                            let Some(value) = value else {
                                return Some(Err(Error::InvalidFormat(
                                    index,
                                    TokenType::Value,
                                    TokenType::Unit,
                                )));
                            };

                            Step::Sub(value, unit)
                        }
                        Operator::Floor => {
                            // value is already checked in the Token::Value branch
                            Step::Floor(unit)
                        }
                    }));
                }
                None => {
                    if operator.is_some() && !was_first {
                        return Some(Err(Error::InvalidFormat(
                            self.tokens.text.len(),
                            TokenType::Value,
                            TokenType::None,
                        )));
                    }
                    return None;
                }
            }
        }
    }
}
