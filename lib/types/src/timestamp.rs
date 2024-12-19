use anyhow::Result;

use chrono::{DateTime, FixedOffset, TimeZone, Utc};

#[derive(Debug, PartialEq)]
pub struct Timestamp(DateTime<FixedOffset>);

impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now().fixed_offset())
    }

    pub fn from_unix_millis(v: i64) -> Option<Self> {
        DateTime::from_timestamp_millis(v).map(|v| v.fixed_offset().into())
    }

    pub fn parse(input: &str, format: &str) -> Result<Self> {
        Ok(DateTime::parse_from_str(input, format)?.into())
    }

    pub fn millisecond(&self) -> i64 {
        self.0.timestamp_subsec_millis() as i64
    }

    pub fn format(&self, format: &str) -> String {
        self.0.format(format).to_string()
    }

    pub fn with_timezone<TZ: TimeZone>(&self, tz: TZ) -> Self {
        self.0.with_timezone(&tz).fixed_offset().into()
    }
}

impl From<DateTime<FixedOffset>> for Timestamp {
    fn from(value: DateTime<FixedOffset>) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_timezone() {
        let ts = Timestamp::from_unix_millis(75600000).unwrap();
        assert_eq!("01.01.1970", ts.format("%d.%m.%Y"));
        assert_eq!(
            "02.01.1970",
            ts.with_timezone(chrono_tz::Europe::Moscow)
                .format("%d.%m.%Y")
        );
    }
}
