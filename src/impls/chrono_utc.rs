use chrono::{DateTime, Utc};

use crate::{time_components::TimeComponents, MathConvertable, Result};

impl MathConvertable for DateTime<Utc> {
    fn now() -> Self {
        Utc::now()
    }

    fn to_components(&self) -> TimeComponents {
        super::chrono_shared::to_components(self)
    }

    fn from_components(components: TimeComponents) -> Result<Self> {
        super::chrono_shared::from_components(components, Utc)
    }
}
