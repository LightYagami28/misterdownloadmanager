//! Date and time helpers over the maintained `jiff` crate.
//!
//! MDM used `chrono` before it was declared unmaintained (RUSTSEC-2026-0543). These modules
//! keep the handful of call patterns MDM needs, backed by `jiff` throughout.

mod duration;
mod format;
mod types;
mod zones;

pub use duration::Duration;
pub use format::{Formatted, SecondsFormat};
pub(crate) use format::FormattedValue;
pub use types::{DateTime, NaiveDate, NaiveDateTime, NaiveTime};
pub use zones::{Datelike, FixedOffset, Local, LocalResult, TimeZone, Timelike, Utc, Weekday};

use std::time::SystemTime;

use jiff::{Timestamp, ZonedDateTime, civil, tz::TimeZone as JiffTimeZone};

pub(crate) fn utc_ts(dt: civil::DateTime) -> Timestamp {
    dt.to_zoned_datetime(JiffTimeZone::UTC)
        .expect("UTC civil times are always unambiguous")
        .timestamp()
}

pub fn now_zoned() -> ZonedDateTime {
    Timestamp::now().to_zoned_datetime(JiffTimeZone::system())
}

pub fn system_time_to_timestamp(t: SystemTime) -> Option<Timestamp> {
    Timestamp::try_from(t).ok()
}
