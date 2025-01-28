use chrono::{DateTime, Datelike, NaiveDate, Timelike};

use crate::{
    error::{Error, Result},
    time_components::TimeComponents,
    time_constants::{SECONDS_PER_HOUR, SECONDS_PER_MINUTE},
};

#[allow(clippy::as_conversions)] // All as conversions are known to succeed
pub fn to_components<Tz: chrono::TimeZone>(datetime: &DateTime<Tz>) -> TimeComponents {
    let datetime = datetime.naive_utc();

    let mut comp = TimeComponents {
        years: datetime.year(),
        days: datetime.day0(),
        seconds: datetime.hour() * SECONDS_PER_HOUR
            + datetime.minute() * SECONDS_PER_MINUTE
            + datetime.second(),
        nanos: datetime.nanosecond(),
    };

    comp.days += comp.days_in_year().2[datetime.month0() as usize];

    comp
}

#[allow(clippy::as_conversions)] // All as conversions are known to succeed
pub fn from_components<Tz: chrono::TimeZone>(
    components: TimeComponents,
    tz: Tz::Offset,
) -> Result<DateTime<Tz>> {
    let (months, days) = components.split_months_days();
    let (hours, minutes, seconds) = components.split_hour_minute_second();

    let TimeComponents {
        years,
        days: _,
        seconds: _,
        nanos,
    } = components;

    let from = || {
        NaiveDate::from_ymd_opt(years, months, days)?
            .and_hms_nano_opt(hours, minutes, seconds, nanos)
    };

    Ok(DateTime::from_naive_utc_and_offset(
        from().ok_or(Error::InvalidTimestamp(components))?,
        tz,
    ))
}
