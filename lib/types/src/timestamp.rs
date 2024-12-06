use chrono::{DateTime, FixedOffset, Utc};

#[derive(Debug)]
pub struct Timestamp(DateTime<FixedOffset>);

impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now().fixed_offset())
    }

    pub fn from_unix_millis(v: i64) -> Option<Self> {
        DateTime::from_timestamp_millis(v).map(|v| v.fixed_offset().into())
    }

    pub fn millisecond(&self) -> i64 {
        self.0.timestamp_subsec_millis() as i64
    }
}

impl From<DateTime<FixedOffset>> for Timestamp {
    fn from(value: DateTime<FixedOffset>) -> Self {
        Self(value)
    }
}
