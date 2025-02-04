use chrono::{DateTime, Local};

use crate::HasNow;

impl HasNow for DateTime<Local> {
    fn now() -> Self {
        Local::now()
    }
}
