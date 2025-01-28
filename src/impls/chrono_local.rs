use chrono::{DateTime, Local};

use crate::RelativeTimeNow;

impl RelativeTimeNow for DateTime<Local> {
    fn now() -> Self {
        Local::now()
    }
}
