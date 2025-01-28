use chrono::{DateTime, Utc};

use crate::RelativeTimeNow;

impl RelativeTimeNow for DateTime<Utc> {
    fn now() -> Self {
        Utc::now()
    }
}
