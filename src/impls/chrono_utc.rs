use chrono::{DateTime, Utc};

use crate::HasNow;

impl HasNow for DateTime<Utc> {
    fn now() -> Self {
        Utc::now()
    }
}
