use chrono::{DateTime, Datelike, NaiveDate, Timelike};

use crate::{
    error::{Error, Result},
    time_components::TimeComponents,
    time_constants::{SECONDS_PER_HOUR, SECONDS_PER_MINUTE},
    HasTimeComponents,
};

impl<Tz: chrono::TimeZone> HasTimeComponents for DateTime<Tz> {
    type AdditionalData = Tz::Offset;

    #[allow(clippy::as_conversions)] // All as conversions are known to succeed
    fn to_components(&self) -> (TimeComponents, Self::AdditionalData) {
        let datetime = self.naive_utc();

        let mut comp = TimeComponents {
            years: datetime.year(),
            days: datetime.day0(),
            seconds: datetime.hour() * SECONDS_PER_HOUR
                + datetime.minute() * SECONDS_PER_MINUTE
                + datetime.second(),
            nanos: datetime.nanosecond(),
        };

        comp.days += comp.days_in_year().2[datetime.month0() as usize];

        (comp, self.offset().clone())
    }

    #[allow(clippy::as_conversions)] // All as conversions are known to succeed
    fn from_components(components: TimeComponents, data: Self::AdditionalData) -> Result<Self> {
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

        Ok(Self::from_naive_utc_and_offset(
            from().ok_or(Error::InvalidTimestamp(components))?,
            data,
        ))
    }
}
