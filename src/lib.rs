pub mod error;
pub mod impls;
pub mod parsing;
pub mod time_components;
pub mod time_constants;

pub use error::{Error, Result};
use parsing::{next_step, Step, Unit, Value};
pub use time_components::MathConvertable;
use time_components::TimeComponents;

pub fn parse_str<T: MathConvertable>(mut s: &str) -> Result<T> {
    s = s.trim();

    // Only grab the now timestamps once, as this might be expensive and we
    // want `now-now` to always resolve to `0`.
    let now = T::now();

    if s == "n" || s == "now" {
        // shortcut so we don't have to transform two-ways
        return Ok(now);
    }

    let mut first = true;
    let mut time = TimeComponents::ZERO;

    while let Some(step) = next_step(&mut s, &mut first)? {
        match step {
            Step::Add(value, unit) => match value {
                Value::Now => {
                    time += now.to_components();
                }
                Value::Number(value) => match unit {
                    Unit::Year => time.add_years(value),
                    Unit::Month => time.add_months(value),
                    Unit::Week => time.add_weeks(value),
                    Unit::Day => time.add_days(value),
                    Unit::Hour => time.add_hours(value),
                    Unit::Minute => time.add_minutes(value),
                    Unit::Second => time.add_seconds(value),
                },
            },
            Step::Sub(value, unit) => match value {
                Value::Now => {
                    time -= now.to_components();
                }
                Value::Number(value) => match unit {
                    Unit::Year => time.sub_years(value),
                    Unit::Month => time.sub_months(value),
                    Unit::Week => time.sub_weeks(value),
                    Unit::Day => time.sub_days(value),
                    Unit::Hour => time.sub_hours(value),
                    Unit::Minute => time.sub_minutes(value),
                    Unit::Second => time.sub_seconds(value),
                },
            },
            Step::Floor(unit) => match unit {
                Unit::Year => time.floor_years(),
                Unit::Month => time.floor_months(),
                Unit::Week => time.floor_weeks(),
                Unit::Day => time.floor_days(),
                Unit::Hour => time.floor_hours(),
                Unit::Minute => time.floor_minutes(),
                Unit::Second => time.floor_seconds(),
            },
        }
    }

    T::from_components(time)
}
