use std::{iter::Peekable, str::CharIndices};

use crate::{Error, Result};

macro_rules! bail {
    ($err:expr) => {
        return Some(Err($err))
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Now,
    Value(u32),
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
}

pub struct Lexer<'s> {
    pub(crate) text: &'s str,
    chars: Peekable<CharIndices<'s>>,
}

impl<'s> Lexer<'s> {
    pub fn new(text: &'s str) -> Self {
        Self {
            text,
            chars: text.char_indices().peekable(),
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<(usize, Token)>;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, c) = self.chars.next()?;
        Some(Ok((
            index,
            match c {
                '0'..='9' => {
                    let mut index_end = index + 1;
                    while matches!(self.chars.peek(), Some((_, '0'..='9'))) {
                        index_end += 1;
                        self.chars.next();
                    }
                    match self.text[index..index_end].parse::<u32>() {
                        Ok(number) => Token::Value(number),
                        Err(err) => bail!(Error::InvalidNumber(
                            self.text[index..index_end].to_string(),
                            err
                        )),
                    }
                }
                'n' => {
                    match self.chars.next() {
                        Some((_, 'o')) => {}
                        Some((index, c)) => bail!(Error::UnexpectedCharacter(index, c)),
                        None => bail!(Error::UnexpectedCharacter(index + 1, '\u{3}')), // 3 is EOT
                    }
                    match self.chars.next() {
                        Some((_, 'w')) => {}
                        Some((index, c)) => bail!(Error::UnexpectedCharacter(index, c)),
                        None => bail!(Error::UnexpectedCharacter(index + 1, '\u{3}')), // 3 is EOT
                    }
                    Token::Now
                }
                '/' => Token::Floor,
                '+' => Token::Add,
                '-' => Token::Sub,
                'y' => Token::Year,
                'M' => Token::Month,
                'w' => Token::Week,
                'd' => Token::Day,
                'h' => Token::Hour,
                'm' => Token::Minute,
                's' => Token::Second,
                c if c.is_whitespace() => return self.next(),
                c => bail!(Error::UnexpectedCharacter(index, c)),
            },
        )))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[inline]
    fn parse(input: &str) -> Result<Vec<(usize, Token)>, Error> {
        Lexer::new(input).collect::<Result<Vec<_>, _>>()
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
    fn now_add_year() {
        parse_eq!(
            "now+1y",
            Ok(vec![
                (0, Token::Now),
                (3, Token::Add),
                (4, Token::Value(1)),
                (5, Token::Year),
            ])
        );
    }

    #[test]
    fn now_sub_day() {
        parse_eq!(
            "now-5d",
            Ok(vec![
                (0, Token::Now),
                (3, Token::Sub),
                (4, Token::Value(5)),
                (5, Token::Day),
            ])
        );
    }

    #[test]
    fn now_floor_week() {
        parse_eq!(
            "now/w",
            Ok(vec![(0, Token::Now), (3, Token::Floor), (4, Token::Week)])
        );
    }

    #[test]
    fn very_large_number() {
        parse_eq!(
            "now+4294967295y",
            Ok(vec![
                (0, Token::Now),
                (3, Token::Add),
                (4, Token::Value(u32::MAX)),
                (14, Token::Year),
            ])
        );
    }

    #[test]
    fn large_number_error() {
        parse_matches!("now+4294967297y", Err(Error::InvalidNumber(_, _)));
    }

    #[test]
    fn invalid_input() {
        parse_matches!("(´･ω･`)", Err(Error::UnexpectedCharacter(0, '(')));
    }

    #[test]
    fn no_input() {
        parse_eq!("", Ok(vec![]));
    }

    #[test]
    fn now_minus_now() {
        parse_eq!(
            "now-now",
            Ok(vec![(0, Token::Now), (3, Token::Sub), (4, Token::Now)])
        );
    }

    #[test]
    fn cursed() {
        // The tokenizer aggressively doesn't care about the structure of the input (as
        // it should)
        parse_eq!(
            "now+-//nownow1nowmMm",
            Ok(vec![
                (0, Token::Now),
                (3, Token::Add),
                (4, Token::Sub),
                (5, Token::Floor),
                (6, Token::Floor),
                (7, Token::Now),
                (10, Token::Now),
                (13, Token::Value(1)),
                (14, Token::Now),
                (17, Token::Minute),
                (18, Token::Month),
                (19, Token::Minute),
            ])
        );
    }
}
