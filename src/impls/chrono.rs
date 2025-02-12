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
        let (months0, days0) = components.split_months_days();
        let (hours, minutes, seconds) = components.split_hours_minutes_seconds();

        let TimeComponents {
            years,
            days: _,
            seconds: _,
            nanos,
        } = components;

        let from = || {
            NaiveDate::from_ymd_opt(years, months0 + 1, days0 + 1)?
                .and_hms_nano_opt(hours, minutes, seconds, nanos)
        };

        Ok(Self::from_naive_utc_and_offset(
            from().ok_or(Error::InvalidTimestamp(components))?,
            data,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time_components::TimeComponents;
    use chrono::{DateTime, FixedOffset, Utc};

    // Tests that the components can be converted into a DateTime object with a Timezone and back
    // to components without any information changing. Also asserts that the resulting DateTime
    // matches an expected string value.
    macro_rules! test_conversion {
        ($components:expr, $offset_ty:ty, $offset:expr, $datetime:expr) => {
            let components = $components;
            let offset = $offset;
            let dt: DateTime<$offset_ty> =
                DateTime::from_components(components.clone(), offset).unwrap();

            assert_eq!(&dt.to_rfc3339(), $datetime);

            let (components_out, offset_out) = dt.to_components();

            assert_eq!(components, components_out);
            assert_eq!(offset, offset_out);
        };
    }
    #[test]
    fn test_new_year() {
        let components = TimeComponents {
            years: 2023,
            days: 0,
            seconds: 0,
            nanos: 0,
        };

        test_conversion!(components, Utc, Utc, "2023-01-01T00:00:00+00:00");
    }

    #[test]
    fn end_of_the_month() {
        let components = TimeComponents {
            years: 2023,
            days: 30, // 0-indexed so this is the 31st
            seconds: 23 * 60 * 60 + 59 * 60 + 59,
            nanos: 0,
        };

        test_conversion!(components, Utc, Utc, "2023-01-31T23:59:59+00:00");
    }

    #[test]
    fn convert_to_and_from_components() {
        let components = TimeComponents {
            years: 2023,
            days: 232,
            seconds: 13200,
            nanos: 0,
        };

        let offset = FixedOffset::east_opt(2 * 60 * 60).unwrap();

        test_conversion!(components, FixedOffset, offset, "2023-08-21T05:40:00+02:00");
    }

    #[test]
    fn seconds_overflow() {
        let components = TimeComponents {
            years: 2023,
            days: 0,
            seconds: 24 * 60 * 60, // ERR, there are not this many seconds in a day
            nanos: 0,
        };

        let datetime: Result<DateTime<Utc>> = DateTime::from_components(components.clone(), Utc);
        assert_eq!(datetime, Err(Error::InvalidTimestamp(components)));
    }
}
