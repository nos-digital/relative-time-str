use crate::time_constants::{
    DAYS_IN_COMMON_MONTH, DAYS_IN_LEAP_MONTH, DAYS_IN_YEAR_COMMON, DAYS_IN_YEAR_LEAP,
    DAYS_PER_WEEK, DAYS_SINCE_COMMON_MONTH, DAYS_SINCE_LEAP_MONTH, MONTHS_PER_YEAR,
    NANOS_PER_SECOND, SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE,
};

#[derive(Debug, Clone)]
pub struct TimeComponents {
    /// The number of years relative to CE.
    pub years: i32,
    /// The number of days since the last year boundary
    pub days: u32,
    /// The number of seconds since the last day boundary
    pub seconds: u32,
    /// The number of nanoseconds since the last second boundary.
    ///
    /// In event of a leap second this may exceed `999_999_999`.
    pub nanos: u32,
}

impl TimeComponents {
    pub const ZERO: Self = Self {
        years: 0,
        days: 0,
        seconds: 0,
        nanos: 0,
    };

    pub fn is_leap_year(&self) -> bool {
        let rem_4 = self.years % 4 == 0;
        let rem_100 = self.years % 100 == 0;
        let rem_400 = self.years % 400 == 0;
        rem_4 && (rem_400 || !rem_100)
    }

    #[inline]
    /// Returns the day information about this year. Returns the total count,
    /// per-month count, aggregated per-month count.
    pub fn days_in_year(&self) -> (u32, [u32; 12], [u32; 12]) {
        if self.is_leap_year() {
            (DAYS_IN_YEAR_LEAP, DAYS_IN_LEAP_MONTH, DAYS_SINCE_LEAP_MONTH)
        } else {
            (
                DAYS_IN_YEAR_COMMON,
                DAYS_IN_COMMON_MONTH,
                DAYS_SINCE_COMMON_MONTH,
            )
        }
    }

    #[inline]
    #[allow(clippy::as_conversions)] // known to never fail
    pub fn split_months_days(&self) -> (u32, u32) {
        let (_, _, day_count_sum) = self.days_in_year();

        let month_in_year = day_count_sum.partition_point(|sum| *sum < self.days) as u32 - 1;
        let day_in_month = self.days - day_count_sum[month_in_year as usize];

        (month_in_year, day_in_month)
    }

    #[inline]
    pub fn split_hour_minute_second(&self) -> (u32, u32, u32) {
        let mut seconds = self.seconds;

        let hours = seconds / SECONDS_PER_HOUR;
        seconds -= hours * SECONDS_PER_HOUR;

        let minutes = seconds / SECONDS_PER_MINUTE;
        seconds -= minutes * SECONDS_PER_HOUR;

        (hours, minutes, seconds)
    }

    #[inline]
    pub fn add_years(&mut self, years: u32) {
        let years: i32 = years.try_into().expect("year is unreasonably high");
        self.years += years;
    }

    #[inline]
    #[allow(clippy::as_conversions)] // known to never fail
    pub fn add_months(&mut self, months: u32) {
        let (mut self_months, days) = self.split_months_days();

        self.add_years(months / MONTHS_PER_YEAR);
        self_months += months % MONTHS_PER_YEAR;

        if self_months > MONTHS_PER_YEAR {
            self_months -= MONTHS_PER_YEAR;
            self.years += 1;
        }

        let (_, day_count, day_count_sum) = self.days_in_year();
        let day_count = day_count[self_months as usize];
        let day_count_sum = day_count_sum[self_months as usize];

        self.days = day_count_sum + days.min(day_count);
    }

    #[inline]
    pub fn add_weeks(&mut self, weeks: u32) {
        self.add_days(weeks * DAYS_PER_WEEK);
    }

    #[inline]
    pub fn add_days(&mut self, days: u32) {
        if days == 0 {
            return;
        }

        self.days += days;

        loop {
            let days_in_year = self.days_in_year().0;
            if self.days < days_in_year {
                return;
            }
            self.years += 1;
            self.days -= days_in_year;
        }
    }

    #[inline]
    pub fn add_hours(&mut self, hours: u32) {
        self.add_seconds(hours * SECONDS_PER_HOUR);
    }

    #[inline]
    pub fn add_minutes(&mut self, minutes: u32) {
        self.add_seconds(minutes * SECONDS_PER_MINUTE);
    }

    #[inline]
    pub fn add_seconds(&mut self, seconds: u32) {
        if seconds == 0 {
            return;
        }
        let seconds = self.seconds + seconds;
        self.seconds = seconds % SECONDS_PER_DAY;
        self.add_days(seconds / SECONDS_PER_DAY);
    }

    #[inline]
    pub fn add_nanos(&mut self, nanos: u32) {
        if nanos == 0 {
            return;
        }
        let nanos = self.nanos + nanos;
        self.nanos = nanos % NANOS_PER_SECOND;
        self.add_seconds(nanos / NANOS_PER_SECOND);
    }

    #[inline]
    pub fn sub_years(&mut self, years: u32) {
        let years: i32 = years.try_into().expect("year is unreasonably high");
        self.years -= years;
    }

    #[inline]
    #[allow(clippy::as_conversions)] // known to never fail
    pub fn sub_months(&mut self, months: u32) {
        let (mut self_months, days) = self.split_months_days();

        self.sub_years(months / MONTHS_PER_YEAR);
        let months = months % MONTHS_PER_YEAR;

        if self_months < months {
            self_months += MONTHS_PER_YEAR;
            self.years -= 1;
        }

        self_months -= months;

        let (_, day_count, day_count_sum) = self.days_in_year();
        let day_count = day_count[self_months as usize];
        let day_count_sum = day_count_sum[self_months as usize];

        self.days = day_count_sum + days.min(day_count);
    }

    #[inline]
    pub fn sub_weeks(&mut self, weeks: u32) {
        self.sub_days(weeks * DAYS_PER_WEEK);
    }

    #[inline]
    pub fn sub_days(&mut self, days: u32) {
        if days == 0 {
            return;
        }

        while self.days < days {
            self.days += self.days_in_year().0;
            self.years -= 1;
        }

        self.days -= days;
    }

    #[inline]
    pub fn sub_hours(&mut self, hours: u32) {
        self.sub_seconds(hours * SECONDS_PER_HOUR);
    }

    #[inline]
    pub fn sub_minutes(&mut self, minutes: u32) {
        self.sub_seconds(minutes * SECONDS_PER_MINUTE);
    }

    #[inline]
    pub fn sub_seconds(&mut self, seconds: u32) {
        if seconds == 0 {
            return;
        }

        let mut days = seconds / SECONDS_PER_DAY;
        let seconds = seconds % SECONDS_PER_DAY;

        if self.seconds < seconds {
            self.seconds += SECONDS_PER_DAY;
            days += 1;
        }

        self.seconds -= seconds;
        self.sub_days(days);
    }

    #[inline]
    pub fn sub_nanos(&mut self, nanos: u32) {
        if nanos == 0 {
            return;
        }

        let mut seconds = nanos / NANOS_PER_SECOND;
        let nanos = nanos % NANOS_PER_SECOND;

        if self.nanos < nanos {
            self.nanos += NANOS_PER_SECOND;
            seconds += 1;
        }
        self.nanos -= nanos;
        self.sub_seconds(seconds);
    }

    #[inline]
    pub fn floor_years(&mut self) {
        self.days = 0;
        self.floor_days();
    }

    #[inline]
    pub fn floor_months(&mut self) {
        let (_month, days) = self.split_months_days();
        self.days -= days;
        self.floor_days();
    }

    #[inline]
    pub fn floor_weeks(&mut self) {
        self.days -= self.days % DAYS_PER_WEEK;
        self.floor_days();
    }

    #[inline]
    pub fn floor_days(&mut self) {
        self.seconds = 0;
        self.floor_seconds();
    }

    #[inline]
    pub fn floor_hours(&mut self) {
        self.seconds -= self.seconds & SECONDS_PER_HOUR;
        self.floor_seconds();
    }

    #[inline]
    pub fn floor_minutes(&mut self) {
        self.seconds -= self.seconds & SECONDS_PER_MINUTE;
        self.floor_seconds();
    }

    #[inline]
    pub fn floor_seconds(&mut self) {
        self.nanos = 0;
    }
}

impl std::ops::Neg for TimeComponents {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        self.years = -self.years;
        self
    }
}

impl std::ops::Add for TimeComponents {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}
impl std::ops::Sub for TimeComponents {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl std::ops::AddAssign for TimeComponents {
    fn add_assign(&mut self, mut rhs: Self) {
        if rhs.years.is_negative() {
            rhs.years *= -1;
            *self -= rhs;
        } else {
            let Self {
                years,
                days,
                seconds,
                nanos,
            } = rhs;
            self.add_nanos(nanos);
            self.add_seconds(seconds);
            self.add_days(days);
            self.years += years;
        }
    }
}

impl std::ops::SubAssign for TimeComponents {
    fn sub_assign(&mut self, mut rhs: Self) {
        if rhs.years.is_negative() {
            rhs.years *= -1;
            *self += rhs;
        } else {
            let Self {
                years,
                days,
                seconds,
                nanos,
            } = rhs;
            self.sub_nanos(nanos);
            self.sub_seconds(seconds);
            self.sub_days(days);
            self.years -= years;
        }
    }
}
