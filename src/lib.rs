pub mod error;
pub mod impls;
pub mod lexer;
pub mod parser;

pub use error::{Error, Result};
use parser::{Expression, Parser, Unit};

pub const MONTHS_PER_YEAR: u32 = 12;
pub const DAYS_PER_WEEK: u32 = 7;
pub const HOURS_PER_DAY: u32 = 24;
pub const MINUTES_PER_HOUR: u32 = 60;
pub const SECONDS_PER_MINUTE: u32 = 60;

pub trait RelativeTimeNow {
    fn now() -> Self;
}

pub trait RelativeTime: Clone {
    fn add_years(self, value: u32) -> Result<Self> {
        self.add_months(value * MONTHS_PER_YEAR)
    }
    fn add_months(self, value: u32) -> Result<Self>;
    fn add_weeks(self, value: u32) -> Result<Self> {
        self.add_days(value * DAYS_PER_WEEK)
    }
    fn add_days(self, value: u32) -> Result<Self> {
        self.add_hours(value * HOURS_PER_DAY)
    }
    fn add_hours(self, value: u32) -> Result<Self> {
        self.add_minutes(value * MINUTES_PER_HOUR)
    }
    fn add_minutes(self, value: u32) -> Result<Self> {
        self.add_seconds(value * SECONDS_PER_MINUTE)
    }
    fn add_seconds(self, value: u32) -> Result<Self>;

    fn sub_years(self, value: u32) -> Result<Self> {
        self.sub_months(value * MONTHS_PER_YEAR)
    }
    fn sub_months(self, value: u32) -> Result<Self>;
    fn sub_weeks(self, value: u32) -> Result<Self> {
        self.sub_days(value * DAYS_PER_WEEK)
    }
    fn sub_days(self, value: u32) -> Result<Self> {
        self.sub_hours(value * HOURS_PER_DAY)
    }
    fn sub_hours(self, value: u32) -> Result<Self> {
        self.sub_minutes(value * MINUTES_PER_HOUR)
    }
    fn sub_minutes(self, value: u32) -> Result<Self> {
        self.sub_seconds(value * SECONDS_PER_MINUTE)
    }
    fn sub_seconds(self, value: u32) -> Result<Self>;

    fn floor_years(self) -> Result<Self>;
    fn floor_months(self) -> Result<Self>;
    fn floor_weeks(self) -> Result<Self>;
    fn floor_days(self) -> Result<Self>;
    fn floor_hours(self) -> Result<Self>;
    fn floor_minutes(self) -> Result<Self>;
    fn floor_seconds(self) -> Result<Self>;
}

pub fn parse_str<T: RelativeTime + RelativeTimeNow>(text: &str) -> Result<T> {
    // Only grab the now timestamps once, as this might be expensive, and we
    // want `now-now` to always resolve to `0`.
    parse_str_with_now(text, T::now())
}

pub fn parse_str_with_now<T: RelativeTime>(text: &str, now: T) -> Result<T> {
    if text.trim().trim_start_matches('+').trim_start() == "now" {
        // shortcut so we don't have to any more logic
        return Ok(now);
    }

    let mut parser = Parser::new(text);

    let mut exprs = Vec::new();

    loop {
        match parser.next().transpose()? {
            None => return Err(Error::MissingNow),
            Some(Expression::Now) => break,
            Some(Expression::Floor(_)) => return Err(Error::FloorBeforeNow),
            Some(expr) => exprs.push(expr),
        }
    }

    let mut time = now;

    for expr in exprs.into_iter().map(Ok).chain(parser) {
        time = match expr? {
            Expression::Now => Err(Error::MultipleNow),
            Expression::Add(0, _) | Expression::Sub(0, _) => Ok(time),
            Expression::Add(value, unit) => match unit {
                Unit::Year => time.add_years(value),
                Unit::Month => time.add_months(value),
                Unit::Week => time.add_weeks(value),
                Unit::Day => time.add_days(value),
                Unit::Hour => time.add_hours(value),
                Unit::Minute => time.add_minutes(value),
                Unit::Second => time.add_seconds(value),
            },
            Expression::Sub(value, unit) => match unit {
                Unit::Year => time.sub_years(value),
                Unit::Month => time.sub_months(value),
                Unit::Week => time.sub_weeks(value),
                Unit::Day => time.sub_days(value),
                Unit::Hour => time.sub_hours(value),
                Unit::Minute => time.sub_minutes(value),
                Unit::Second => time.sub_seconds(value),
            },
            Expression::Floor(unit) => match unit {
                Unit::Year => time.floor_years(),
                Unit::Month => time.floor_months(),
                Unit::Week => time.floor_weeks(),
                Unit::Day => time.floor_days(),
                Unit::Hour => time.floor_hours(),
                Unit::Minute => time.floor_minutes(),
                Unit::Second => time.floor_seconds(),
            },
        }?
    }

    Ok(time)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINUTE: i64 = 60;
    const HOUR: i64 = 60 * MINUTE;
    const DAY: i64 = 24 * HOUR;
    const WEEK: i64 = 7 * DAY;

    /// Records what the parser asked for, so the tests don't depend on a
    /// date-time library.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    struct Mock {
        months: i64,
        seconds: i64,
        floors: Vec<Unit>,
    }

    impl Mock {
        fn offset(months: i64, seconds: i64) -> Self {
            Self {
                months,
                seconds,
                floors: Vec::new(),
            }
        }

        fn floor(mut self, unit: Unit) -> Result<Self> {
            self.floors.push(unit);
            Ok(self)
        }
    }

    impl RelativeTime for Mock {
        fn add_months(mut self, value: u32) -> Result<Self> {
            self.months += i64::from(value);
            Ok(self)
        }

        fn add_seconds(mut self, value: u32) -> Result<Self> {
            self.seconds += i64::from(value);
            Ok(self)
        }

        fn sub_months(mut self, value: u32) -> Result<Self> {
            self.months -= i64::from(value);
            Ok(self)
        }

        fn sub_seconds(mut self, value: u32) -> Result<Self> {
            self.seconds -= i64::from(value);
            Ok(self)
        }

        fn floor_years(self) -> Result<Self> {
            self.floor(Unit::Year)
        }

        fn floor_months(self) -> Result<Self> {
            self.floor(Unit::Month)
        }

        fn floor_weeks(self) -> Result<Self> {
            self.floor(Unit::Week)
        }

        fn floor_days(self) -> Result<Self> {
            self.floor(Unit::Day)
        }

        fn floor_hours(self) -> Result<Self> {
            self.floor(Unit::Hour)
        }

        fn floor_minutes(self) -> Result<Self> {
            self.floor(Unit::Minute)
        }

        fn floor_seconds(self) -> Result<Self> {
            self.floor(Unit::Second)
        }
    }

    macro_rules! test_input_string {
        ($input:expr, $output:expr) => {
            let res = parse_str_with_now($input, Mock::default()).expect("input invalid");
            assert_eq!(res, $output);
        };
    }

    #[test]
    fn literal_now() {
        test_input_string!("now", Mock::default());
    }

    #[test]
    fn unary_now() {
        test_input_string!("+now", Mock::default());
        test_input_string!("  +    now", Mock::default());
    }

    #[test]
    fn now_plus_days() {
        test_input_string!("now + 1d", Mock::offset(0, DAY));
        test_input_string!("now + 3d", Mock::offset(0, 3 * DAY));
        test_input_string!("now + 30d", Mock::offset(0, 30 * DAY));
    }

    #[test]
    fn now_minus_hours() {
        test_input_string!("now - 2h", Mock::offset(0, -2 * HOUR));
    }

    #[test]
    fn now_plus_1w() {
        test_input_string!("now + 1w", Mock::offset(0, WEEK));
    }

    #[test]
    fn now_plus_2mo() {
        test_input_string!("now + 2M", Mock::offset(2, 0));
    }

    #[test]
    fn now_minus_1y() {
        test_input_string!("now - 1y", Mock::offset(-12, 0));
    }

    #[test]
    fn now_with_multiple_offsets() {
        test_input_string!(
            "now + 1d - 2h + 30m",
            Mock::offset(0, DAY - 2 * HOUR + 30 * MINUTE)
        );
    }

    #[test]
    fn now_with_whitespace_variants() {
        test_input_string!("now+1d", Mock::offset(0, DAY));
        test_input_string!("now    +    1d", Mock::offset(0, DAY));
        test_input_string!("now+   1d", Mock::offset(0, DAY));
    }

    #[test]
    fn leading_zeroes() {
        test_input_string!("now + 00015s", Mock::offset(0, 15));
    }

    #[test]
    fn now_with_with_overflow_addition() {
        test_input_string!("now + 90s", Mock::offset(0, 90));
        test_input_string!("now + 61m", Mock::offset(0, 61 * MINUTE));
    }

    #[test]
    fn now_zero_offset() {
        test_input_string!("now + 0s", Mock::default());
        test_input_string!("now - 0d", Mock::default());
    }

    #[test]
    fn now_negative_offsets_combined() {
        test_input_string!("now - 1d - 2h", Mock::offset(0, -DAY - 2 * HOUR));
    }

    #[test]
    fn complex_expression() {
        test_input_string!(
            "now + 1y - 1M + 3w - 2d + 4h - 30m + 15s",
            Mock::offset(11, 3 * WEEK - 2 * DAY + 4 * HOUR - 30 * MINUTE + 15)
        );
    }

    #[test]
    fn floor_after_offset() {
        test_input_string!(
            "now - 1d/d",
            Mock {
                floors: vec![Unit::Day],
                ..Mock::offset(0, -DAY)
            }
        );
    }
}
