use chrono::{DateTime, Datelike, Days, DurationRound, Months, TimeDelta};

use crate::{
    MONTHS_PER_YEAR, RelativeTime,
    error::{Error, Result},
};

impl<Tz: chrono::TimeZone> RelativeTime for DateTime<Tz> {
    fn add_months(self, value: u32) -> Result<Self> {
        let delta = Months::new(value);
        self.checked_add_months(delta)
            .ok_or(Error::InvalidTimestamp)
    }

    fn add_days(self, value: u32) -> Result<Self> {
        let delta = Days::new(value.into());
        self.checked_add_days(delta).ok_or(Error::InvalidTimestamp)
    }

    fn add_seconds(self, value: u32) -> Result<Self> {
        let delta = TimeDelta::try_seconds(value.into()).ok_or(Error::InvalidDelta)?;
        self.checked_add_signed(delta)
            .ok_or(Error::InvalidTimestamp)
    }

    fn sub_months(self, value: u32) -> Result<Self> {
        let delta = Months::new(value);
        self.checked_sub_months(delta)
            .ok_or(Error::InvalidTimestamp)
    }

    fn sub_days(self, value: u32) -> Result<Self> {
        let delta = Days::new(value.into());
        self.checked_sub_days(delta).ok_or(Error::InvalidTimestamp)
    }

    fn sub_seconds(self, value: u32) -> Result<Self> {
        let delta = TimeDelta::try_seconds(value.into()).ok_or(Error::InvalidDelta)?;
        self.checked_sub_signed(delta)
            .ok_or(Error::InvalidTimestamp)
    }

    fn floor_years(self) -> Result<Self> {
        let offset = self.offset().clone();
        let years = self
            .to_utc()
            .years_since(Self::MIN_UTC)
            .expect("min is used as base");
        let datetime = chrono::NaiveDateTime::MIN + Months::new(years * MONTHS_PER_YEAR);
        Ok(Self::from_naive_utc_and_offset(datetime, offset))
    }

    fn floor_months(self) -> Result<Self> {
        let offset = self.offset().clone();
        let utc = self.to_utc();
        let years = utc.years_since(Self::MIN_UTC).expect("min is used as base");
        let months = utc.month0();
        let datetime = chrono::NaiveDateTime::MIN + Months::new(years * MONTHS_PER_YEAR + months);
        Ok(Self::from_naive_utc_and_offset(datetime, offset))
    }

    fn floor_weeks(self) -> Result<Self> {
        let delta = TimeDelta::weeks(1);
        self.duration_trunc(delta)
            .map_err(|_err| Error::InvalidTimestamp)
    }

    fn floor_days(self) -> Result<Self> {
        let delta = TimeDelta::days(1);
        self.duration_trunc(delta)
            .map_err(|_err| Error::InvalidTimestamp)
    }

    fn floor_hours(self) -> Result<Self> {
        let delta = TimeDelta::hours(1);
        self.duration_trunc(delta)
            .map_err(|_err| Error::InvalidTimestamp)
    }

    fn floor_minutes(self) -> Result<Self> {
        let delta = TimeDelta::minutes(1);
        self.duration_trunc(delta)
            .map_err(|_err| Error::InvalidTimestamp)
    }

    fn floor_seconds(self) -> Result<Self> {
        let delta = TimeDelta::seconds(1);
        self.duration_trunc(delta)
            .map_err(|_err| Error::InvalidTimestamp)
    }
}
