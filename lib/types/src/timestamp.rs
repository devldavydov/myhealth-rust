use anyhow::{anyhow, Result};

use chrono::{DateTime, FixedOffset, NaiveDateTime, NaiveTime, TimeZone, Utc};

#[derive(Debug, PartialEq)]
pub struct Timestamp(DateTime<FixedOffset>);

impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now().fixed_offset())
    }

    pub fn from_unix_millis(v: i64) -> Option<Self> {
        DateTime::from_timestamp_millis(v).map(|v| v.fixed_offset().into())
    }

    pub fn parse_date<TZ: TimeZone>(input: &str, format: &str, tz: TZ) -> Result<Self> {
        let dt = NaiveDateTime::parse_from_str(
            &format!("{input}T00:00:00"),
            &format!("{format}T%H:%M:%S"),
        )?;
        tz.from_local_datetime(&dt)
            .single()
            .ok_or(anyhow!("bad date"))
            .map(|v| v.fixed_offset().into())
    }

    pub fn unix_millis(&self) -> i64 {
        self.0.timestamp_millis()
    }

    pub fn format(&self, format: &str) -> String {
        self.0.format(format).to_string()
    }

    pub fn with_timezone<TZ: TimeZone>(&self, tz: TZ) -> Self {
        self.0.with_timezone(&tz).fixed_offset().into()
    }

    pub fn start_of_day(&self) -> Self {
        self.0.with_time(NaiveTime::MIN).unwrap().into()
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
    use anyhow::Result;

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

    #[test]
    fn test_parse_date() -> Result<()> {
        assert_eq!(
            1734728400000,
            Timestamp::parse_date("21.12.2024", "%d.%m.%Y", chrono_tz::Europe::Moscow)?
                .unix_millis()
        );
        assert_eq!(
            1734739200000,
            Timestamp::parse_date("21.12.2024", "%d.%m.%Y", chrono_tz::UTC)?.unix_millis()
        );

        Ok(())
    }
}
