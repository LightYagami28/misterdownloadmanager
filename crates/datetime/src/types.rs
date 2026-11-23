//! Date and time value types.

use std::marker::PhantomData;

use jiff::{Timestamp, civil, tz::TimeZone as JiffTimeZone};

use crate::{
    Duration, FixedOffset, Formatted, FormattedValue, Local, SecondsFormat, Utc, utc_ts,
};

/// A timestamp in an instant-like time zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime<Tz> {
    pub(crate) ts: Timestamp,
    zone: PhantomData<Tz>,
}

impl<Tz> DateTime<Tz> {
    pub fn timestamp(&self) -> i64 {
        self.ts.as_second()
    }

    pub fn timestamp_millis(&self) -> i64 {
        self.ts.as_millisecond()
    }

    pub fn timestamp_subsec_millis(&self) -> u32 {
        self.ts.as_millisecond().rem_euclid(1_000) as u32
    }

    pub fn naive_utc(&self) -> NaiveDateTime {
        NaiveDateTime(self.ts.to_zoned_datetime(JiffTimeZone::UTC).datetime())
    }

    pub fn naive_local(&self) -> NaiveDateTime {
        NaiveDateTime(self.ts.to_zoned_datetime(JiffTimeZone::system()).datetime())
    }

    pub fn offset(&self) -> FixedOffset {
        FixedOffset(self.ts.to_zoned_datetime(JiffTimeZone::system()).offset().seconds())
    }

    pub fn with_timezone<Tz2>(&self, _tz: &Tz2) -> DateTime<Tz2> {
        DateTime { ts: self.ts, zone: PhantomData }
    }

    pub fn and_utc(&self) -> DateTime<Utc> {
        self.with_timezone(&Utc)
    }

    pub fn date(&self) -> NaiveDate {
        self.naive_local().date()
    }

    pub fn format(&self, fmt: &str) -> Formatted<'_> {
        Formatted::new(fmt, FormattedValue::Timestamp(&self.ts))
    }

    pub fn elapsed(&self) -> std::time::Duration {
        let now = Timestamp::now();
        let mut secs = now.as_second() - self.ts.as_second();
        let mut nanos = now.subsec_nanosecond() - self.ts.subsec_nanosecond();
        if nanos < 0 {
            secs -= 1;
            nanos += 1_000_000_000;
        }
        std::time::Duration::new(secs.max(0) as u64, nanos as u32)
    }

    pub fn to_rfc3339(&self) -> String {
        self.ts.to_string()
    }

    pub fn to_rfc3339_opts(&self, _opts: SecondsFormat, _use_z: bool) -> String {
        self.ts.to_string()
    }

    pub fn from_timestamp(secs: i64, nsecs: u32) -> Option<Self> {
        Timestamp::from_second(secs)
            .ok()
            .and_then(|ts| ts.checked_add(jiff::SignedDuration::from_nanos(nsecs as i64)).ok())
            .map(Self::from)
    }

    pub fn from_naive_utc_and_offset(naive: NaiveDateTime, _offset: FixedOffset) -> Self {
        Self::from(utc_ts(naive.0))
    }
}

impl DateTime<Utc> {
    pub fn now() -> Self {
        Self::from(Timestamp::now())
    }

    pub fn parse_from_rfc3339(s: &str) -> Result<Self, jiff::Error> {
        s.parse::<Timestamp>().map(Self::from)
    }
}

impl DateTime<Local> {
    pub fn now() -> Self {
        Self::from(Timestamp::now())
    }
}

impl<Tz> From<Timestamp> for DateTime<Tz> {
    fn from(ts: Timestamp) -> Self {
        Self { ts, zone: PhantomData }
    }
}

impl<Tz> std::ops::Add<Duration> for DateTime<Tz> {
    type Output = Self;
    fn add(self, rhs: Duration) -> Self {
        Self::from(self.ts + rhs.get())
    }
}

impl<Tz> std::ops::Sub<Duration> for DateTime<Tz> {
    type Output = Self;
    fn sub(self, rhs: Duration) -> Self {
        Self::from(self.ts - rhs.get())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NaiveDateTime(pub(crate) civil::DateTime);

impl NaiveDateTime {
    pub fn parse_from_str(s: &str, format: &str) -> Result<Self, jiff::Error> {
        civil::DateTime::strptime(format, s).map(Self)
    }

    pub fn date(&self) -> NaiveDate {
        NaiveDate(self.0.date())
    }

    pub fn time(&self) -> NaiveTime {
        NaiveTime(self.0.time())
    }

    pub fn format(&self, fmt: &str) -> Formatted<'_> {
        Formatted::new(fmt, FormattedValue::Civil(&self.0))
    }

    pub fn elapsed(&self) -> std::time::Duration {
        let now = Timestamp::now();
        let then = utc_ts(self.0);
        std::time::Duration::from_secs(now.as_second().saturating_sub(then.as_second()).max(0) as u64)
    }
}

impl std::ops::Sub for NaiveDateTime {
    type Output = Duration;
    fn sub(self, rhs: NaiveDateTime) -> Duration {
        Duration::new(utc_ts(self.0) - utc_ts(rhs.0))
    }
}

impl std::ops::Add<Duration> for NaiveDateTime {
    type Output = NaiveDateTime;
    fn add(self, rhs: Duration) -> Self {
        let ts = utc_ts(self.0) + rhs.get();
        Self(ts.to_zoned_datetime(JiffTimeZone::UTC).datetime())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NaiveDate(pub(crate) civil::Date);

impl NaiveDate {
    pub fn from_ymd_opt(year: i32, month: u32, day: u32) -> Option<Self> {
        if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
            return None;
        }
        let year = i16::try_from(year).ok()?;
        civil::Date::new(year, month as i8, day as i8).ok().map(Self)
    }

    pub fn and_hms_opt(self, h: u32, m: u32, s: u32) -> Option<NaiveDateTime> {
        NaiveTime::from_hms_opt(h, m, s).map(|t| t.on(self))
    }

    pub fn and_time(self, time: NaiveTime) -> NaiveDateTime {
        time.on(self)
    }

    pub fn with_day(self, day: u32) -> Option<Self> {
        civil::Date::new(self.0.year(), self.0.month(), day as i8).ok().map(Self)
    }

    pub fn format(&self, fmt: &str) -> Formatted<'_> {
        Formatted::new(fmt, FormattedValue::Date(&self.0))
    }

    fn at_midnight(self) -> NaiveDateTime {
        self.and_hms_opt(0, 0, 0).expect("midnight is valid")
    }
}

impl std::ops::Add<Duration> for NaiveDate {
    type Output = NaiveDate;
    fn add(self, rhs: Duration) -> Self {
        let ts = utc_ts(self.at_midnight().0) + rhs.get();
        Self(ts.to_zoned_datetime(JiffTimeZone::UTC).date())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NaiveTime(pub(crate) civil::Time);

impl NaiveTime {
    pub fn from_hms_opt(h: u32, m: u32, s: u32) -> Option<Self> {
        if h > 23 || m > 59 || s > 59 {
            return None;
        }
        civil::Time::new(h as i8, m as i8, s as i8, 0).ok().map(Self)
    }

    pub(crate) fn on(self, date: NaiveDate) -> NaiveDateTime {
        NaiveDateTime(date.0.at(
            self.0.hour(),
            self.0.minute(),
            self.0.second(),
            self.0.subsec_nanosecond(),
        ))
    }
}
