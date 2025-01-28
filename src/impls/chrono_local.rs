use chrono::{DateTime, Local};

use crate::{time_components::TimeComponents, MathConvertable, Result};

impl MathConvertable for DateTime<Local> {
    fn now() -> Self {
        Local::now()
    }

    fn to_components(&self) -> TimeComponents {
        super::chrono_shared::to_components(self)
    }

    fn from_components(components: TimeComponents) -> Result<Self> {
        // Have to re-grab the current time to obtain the timezone
        let tz = *Self::now().offset();
        super::chrono_shared::from_components(components, tz)
    }
}
