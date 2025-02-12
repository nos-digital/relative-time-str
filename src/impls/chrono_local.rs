use chrono::{DateTime, Local};

use crate::HasNow;

impl HasNow for DateTime<Local> {
    fn now() -> Self {
        Local::now()
    }
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, Local};
    use crate::parse_str;

    #[test]
    fn parse_test() {
        // ISSUE: this test crashes but I don't think it should
        let _datetime: DateTime<Local> = parse_str("now-55y").unwrap();
    }
}