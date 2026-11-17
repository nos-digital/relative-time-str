use jiff::{RoundMode, Span, Unit, ZonedDateTime, ZonedDateTimeRound};

use crate::{
    RelativeTime,
    error::{Error, Result},
};

/// Months and days are calendar units: `now + 1d` keeps the wall-clock time,
/// even across a DST change. Hours and smaller are exact durations.
///
/// Floors are done in the time zone of the value, and weeks start on Monday.
impl RelativeTime for ZonedDateTime {
    fn add_months(self, value: u32) -> Result<Self> {
        let span = Span::new()
            .try_months(value)
            .map_err(|_err| Error::InvalidDelta)?;
        self.checked_add(span)
            .map_err(|_err| Error::InvalidTimestamp)
    }

    fn add_days(self, value: u32) -> Result<Self> {
        let span = Span::new()
            .try_days(value)
            .map_err(|_err| Error::InvalidDelta)?;
        self.checked_add(span)
            .map_err(|_err| Error::InvalidTimestamp)
    }

    fn add_seconds(self, value: u32) -> Result<Self> {
        let span = Span::new()
            .try_seconds(value)
            .map_err(|_err| Error::InvalidDelta)?;
        self.checked_add(span)
            .map_err(|_err| Error::InvalidTimestamp)
    }

    fn sub_months(self, value: u32) -> Result<Self> {
        let span = Span::new()
            .try_months(value)
            .map_err(|_err| Error::InvalidDelta)?;
        self.checked_sub(span)
            .map_err(|_err| Error::InvalidTimestamp)
    }

    fn sub_days(self, value: u32) -> Result<Self> {
        let span = Span::new()
            .try_days(value)
            .map_err(|_err| Error::InvalidDelta)?;
        self.checked_sub(span)
            .map_err(|_err| Error::InvalidTimestamp)
    }

    fn sub_seconds(self, value: u32) -> Result<Self> {
        let span = Span::new()
            .try_seconds(value)
            .map_err(|_err| Error::InvalidDelta)?;
        self.checked_sub(span)
            .map_err(|_err| Error::InvalidTimestamp)
    }

    fn floor_years(self) -> Result<Self> {
        self.first_of_year()
            .and_then(|zdt| zdt.start_of_day())
            .map_err(|_err| Error::InvalidTimestamp)
    }

    fn floor_months(self) -> Result<Self> {
        self.first_of_month()
            .and_then(|zdt| zdt.start_of_day())
            .map_err(|_err| Error::InvalidTimestamp)
    }

    fn floor_weeks(self) -> Result<Self> {
        let days_since_monday = self.weekday().to_monday_zero_offset();
        self.checked_sub(Span::new().days(days_since_monday))
            .and_then(|zdt| zdt.start_of_day())
            .map_err(|_err| Error::InvalidTimestamp)
    }

    fn floor_days(self) -> Result<Self> {
        self.start_of_day().map_err(|_err| Error::InvalidTimestamp)
    }

    fn floor_hours(self) -> Result<Self> {
        trunc(&self, Unit::Hour)
    }

    fn floor_minutes(self) -> Result<Self> {
        trunc(&self, Unit::Minute)
    }

    fn floor_seconds(self) -> Result<Self> {
        trunc(&self, Unit::Second)
    }
}

fn trunc(zdt: &ZonedDateTime, unit: Unit) -> Result<ZonedDateTime> {
    let options = ZonedDateTimeRound::new()
        .smallest(unit)
        .mode(RoundMode::Trunc);
    zdt.round(options).map_err(|_err| Error::InvalidTimestamp)
}

#[cfg(feature = "jiff-now")]
impl crate::RelativeTimeNow for ZonedDateTime {
    /// The current time in the system time zone.
    fn now() -> Self {
        Self::now()
    }
}

#[cfg(all(test, feature = "jiff-now"))]
mod tests {
    use super::*;
    use crate::parse_str_with_now;

    macro_rules! test_input_string {
        ($now:expr, $input:expr, $output:expr) => {
            let now: ZonedDateTime = $now.parse().expect("now is invalid");
            let res = parse_str_with_now($input, now).expect("input is invalid");
            assert_eq!(res.to_string(), $output);
        };
        ($input:expr, $output:expr) => {
            test_input_string!("2023-08-21T05:40:00[Europe/Amsterdam]", $input, $output);
        };
    }

    #[test]
    fn literal_now() {
        test_input_string!("now", "2023-08-21T05:40:00+02:00[Europe/Amsterdam]");
    }

    #[test]
    fn offsets() {
        test_input_string!("now + 30d", "2023-09-20T05:40:00+02:00[Europe/Amsterdam]");
        test_input_string!("now - 2h", "2023-08-21T03:40:00+02:00[Europe/Amsterdam]");
        test_input_string!("now + 1w", "2023-08-28T05:40:00+02:00[Europe/Amsterdam]");
        test_input_string!("now + 2M", "2023-10-21T05:40:00+02:00[Europe/Amsterdam]");
        test_input_string!("now - 1y", "2022-08-21T05:40:00+02:00[Europe/Amsterdam]");
        test_input_string!("now + 61m", "2023-08-21T06:41:00+02:00[Europe/Amsterdam]");
    }

    #[test]
    fn complex_expression() {
        test_input_string!(
            "now + 1y - 1M + 3w - 2d + 4h - 30m + 15s",
            "2024-08-09T09:10:15+02:00[Europe/Amsterdam]"
        );
    }

    #[test]
    fn month_end_is_clamped() {
        test_input_string!(
            "2023-01-31T12:00:00[Europe/Amsterdam]",
            "now + 1M",
            "2023-02-28T12:00:00+01:00[Europe/Amsterdam]"
        );
    }

    #[test]
    fn days_keep_wall_clock_across_dst() {
        // 2023-10-29 has 25 hours in Amsterdam.
        test_input_string!(
            "2023-10-28T12:00:00[Europe/Amsterdam]",
            "now + 1d",
            "2023-10-29T12:00:00+01:00[Europe/Amsterdam]"
        );
        test_input_string!(
            "2023-10-28T12:00:00[Europe/Amsterdam]",
            "now + 24h",
            "2023-10-29T11:00:00+01:00[Europe/Amsterdam]"
        );
    }

    #[test]
    fn floors() {
        let now = "2023-08-23T05:40:12.5[Europe/Amsterdam]";
        test_input_string!(now, "now/y", "2023-01-01T00:00:00+01:00[Europe/Amsterdam]");
        test_input_string!(now, "now/M", "2023-08-01T00:00:00+02:00[Europe/Amsterdam]");
        test_input_string!(now, "now/w", "2023-08-21T00:00:00+02:00[Europe/Amsterdam]");
        test_input_string!(now, "now/d", "2023-08-23T00:00:00+02:00[Europe/Amsterdam]");
        test_input_string!(now, "now/h", "2023-08-23T05:00:00+02:00[Europe/Amsterdam]");
        test_input_string!(now, "now/m", "2023-08-23T05:40:00+02:00[Europe/Amsterdam]");
        test_input_string!(now, "now/s", "2023-08-23T05:40:12+02:00[Europe/Amsterdam]");
    }

    #[test]
    fn floor_week_on_monday() {
        test_input_string!(
            "2023-08-21T05:40:00[Europe/Amsterdam]",
            "now/w",
            "2023-08-21T00:00:00+02:00[Europe/Amsterdam]"
        );
        test_input_string!(
            "2023-08-27T23:59:00[Europe/Amsterdam]",
            "now/w",
            "2023-08-21T00:00:00+02:00[Europe/Amsterdam]"
        );
    }

    #[test]
    fn now_is_system_time() {
        let before = ZonedDateTime::now();
        let res: ZonedDateTime = crate::parse_str("now").expect("now is valid");
        assert!(before <= res);
        assert_eq!(before.time_zone(), res.time_zone());
    }
}
