use std::{iter::Peekable, str::CharIndices};

use crate::{Error, Result};

macro_rules! bail {
    ($err:expr) => {
        return Some(Err($err))
    };
}

macro_rules! ensure {
    ($if:expr, $err:expr) => {
        if !$if {
            return Some(Err($err));
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
        loop {
            let (index, c) = self.chars.next()?;

            return Some(Ok(match c {
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
                c if c.is_whitespace() => continue,
                c => bail!(Error::InvalidCharacter(index, c)),
            }));
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
                Some(Err(err)) => bail!(err),
                Some(Ok(Token::Value(index, v))) => {
                    ensure!(
                        operator.is_some() && operator != Some(Operator::Floor),
                        Error::InvalidFormat(index, TokenType::Operator, TokenType::Value)
                    );
                    ensure!(
                        value.is_none(),
                        Error::InvalidFormat(index, TokenType::Unit, TokenType::Value)
                    );
                    value = Some(v);
                }
                Some(Ok(Token::Operator(index, o))) => {
                    // Allow +- syntax
                    if operator == Some(Operator::Add) && o == Operator::Sub {
                        operator = None;
                    }
                    ensure!(
                        operator != Some(Operator::Floor),
                        Error::InvalidFormat(index, TokenType::Unit, TokenType::Operator)
                    );
                    ensure!(
                        operator.is_none(),
                        Error::InvalidFormat(index, TokenType::Value, TokenType::Operator)
                    );
                    ensure!(
                        value.is_none(),
                        Error::InvalidFormat(index, TokenType::Unit, TokenType::Operator)
                    );
                    operator = Some(o);
                }
                Some(Ok(Token::Unit(index, unit))) => {
                    let Some(operator) = operator else {
                        bail!(Error::InvalidFormat(
                            index,
                            TokenType::Operator,
                            TokenType::Unit,
                        ));
                    };

                    return Some(Ok(match operator {
                        Operator::Add => {
                            let Some(value) = value else {
                                bail!(Error::InvalidFormat(
                                    index,
                                    TokenType::Value,
                                    TokenType::Unit,
                                ));
                            };

                            Step::Add(value, unit)
                        }
                        Operator::Sub => {
                            let Some(value) = value else {
                                bail!(Error::InvalidFormat(
                                    index,
                                    TokenType::Value,
                                    TokenType::Unit,
                                ));
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
                    ensure!(
                        operator.is_none() || was_first,
                        Error::InvalidFormat(
                            self.tokens.text.len(),
                            TokenType::Value,
                            TokenType::None,
                        )
                    );
                    return None;
                }
            }
        }
    }
}
