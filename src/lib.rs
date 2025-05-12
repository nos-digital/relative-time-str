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
    use chrono::{Local, NaiveDateTime, TimeZone};

    macro_rules! test_input_string {
        ($input:expr, $output:expr) => {
            let naive_datetime =
                NaiveDateTime::parse_from_str("2023-08-21T05:40:00", "%Y-%m-%dT%H:%M:%S")
                    .expect("input date time invalid");
            let local_datetime = Local
                .from_local_datetime(&naive_datetime)
                .single()
                .expect("input date time ambiguous");

            let res = parse_str_with_now($input, local_datetime).expect("input date time invalid");

            assert_eq!(format!("{}", res.format("%Y-%m-%dT%H:%M:%S")), $output);
        };
    }

    #[test]
    fn literal_now() {
        test_input_string!("now", "2023-08-21T05:40:00");
    }

    #[test]
    fn unary_now() {
        test_input_string!("+now", "2023-08-21T05:40:00");
        test_input_string!("  +    now", "2023-08-21T05:40:00");
    }

    #[test]
    fn now_plus_days() {
        test_input_string!("now + 1d", "2023-08-22T05:40:00");
        test_input_string!("now + 3d", "2023-08-24T05:40:00");
        test_input_string!("now + 30d", "2023-09-20T05:40:00");
    }

    #[test]
    fn now_minus_hours() {
        test_input_string!("now - 2h", "2023-08-21T03:40:00");
    }

    #[test]
    fn now_plus_1w() {
        test_input_string!("now + 1w", "2023-08-28T05:40:00");
    }

    #[test]
    fn now_plus_2mo() {
        test_input_string!("now + 2M", "2023-10-21T05:40:00");
    }

    #[test]
    fn now_minus_1y() {
        test_input_string!("now - 1y", "2022-08-21T05:40:00");
    }

    #[test]
    fn now_with_multiple_offsets() {
        test_input_string!("now + 1d - 2h + 30m", "2023-08-22T04:10:00");
    }

    #[test]
    fn now_with_whitespace_variants() {
        test_input_string!("now+1d", "2023-08-22T05:40:00");
        test_input_string!("now    +    1d", "2023-08-22T05:40:00");
        test_input_string!("now+   1d", "2023-08-22T05:40:00");
    }

    #[test]
    fn test_leading_zeroes() {
        test_input_string!("now + 00015s", "2023-08-21T05:40:15");
    }

    #[test]
    fn now_with_with_overflow_addition() {
        test_input_string!("now + 90s", "2023-08-21T05:41:30");
        test_input_string!("now + 61m", "2023-08-21T06:41:00");
    }

    #[test]
    fn now_zero_offset() {
        test_input_string!("now + 0s", "2023-08-21T05:40:00");
        test_input_string!("now - 0d", "2023-08-21T05:40:00");
    }

    #[test]
    fn now_negative_offsets_combined() {
        test_input_string!("now - 1d - 2h", "2023-08-20T03:40:00");
    }

    #[test]
    fn complex_expression() {
        test_input_string!(
            "now + 1y - 1M + 3w - 2d + 4h - 30m + 15s",
            "2024-08-09T09:10:15"
        );
    }
}
