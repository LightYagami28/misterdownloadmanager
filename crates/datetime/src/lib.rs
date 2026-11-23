//! Small date/time helpers over the maintained `jiff` crate.
//!
//! MDM used `chrono` before it was declared unmaintained (RUSTSEC-2026-0543). These types keep
//! the handful of call patterns MDM needs, backed by `jiff` throughout.

use std::{fmt, marker::PhantomData, time::SystemTime};

use jiff::{
    Timestamp, ZonedDateTime, civil,
    fmt::strtime,
    tz::TimeZone as JiffTimeZone,
};

#[derive(Clone, Copy, Debug)]
pub struct Utc;
#[derive(Clone, Copy, Debug)]
pub struct Local;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixedOffset(i32);

/// A timestamp in an instant-like time zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime<Tz> {
    ts: Timestamp,
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
        Formatted { fmt: fmt.to_owned(), value: self.into() }
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

    pub fn parse_from_rfc3339(s: &str) -> Result<Self, jiff::Error> {
        s.parse::<Timestamp>().map(Self::from)
    }

    pub fn from_naive_utc_and_offset(naive: NaiveDateTime, _offset: FixedOffset) -> Self {
        Self::from(utc_ts(naive.0))
    }
}

impl DateTime<Utc> {
    pub fn now() -> Self {
        Self::from(Timestamp::now())
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
        Self::from(self.ts + rhs.0)
    }
}

impl<Tz> std::ops::Sub<Duration> for DateTime<Tz> {
    type Output = Self;
    fn sub(self, rhs: Duration) -> Self {
        Self::from(self.ts - rhs.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NaiveDateTime(civil::DateTime);

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
        Formatted { fmt: fmt.to_owned(), value: self.into() }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        let now = Timestamp::now();
        let then = utc_ts(self.0);
        let secs = now.as_second() - then.as_second();
        std::time::Duration::from_secs(secs.max(0) as u64)
    }
}

impl std::ops::Sub for NaiveDateTime {
    type Output = Duration;
    fn sub(self, rhs: NaiveDateTime) -> Duration {
        Duration(utc_ts(self.0) - utc_ts(rhs.0))
    }
}

impl std::ops::Add<Duration> for NaiveDateTime {
    type Output = NaiveDateTime;
    fn add(self, rhs: Duration) -> Self {
        let ts = utc_ts(self.0) + rhs.0;
        Self(ts.to_zoned_datetime(JiffTimeZone::UTC).datetime())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NaiveDate(civil::Date);

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
        Formatted { fmt: fmt.to_owned(), value: self.into() }
    }

    fn at_midnight(self) -> NaiveDateTime {
        self.and_hms_opt(0, 0, 0).expect("midnight is valid")
    }
}

impl std::ops::Add<Duration> for NaiveDate {
    type Output = NaiveDate;
    fn add(self, rhs: Duration) -> Self {
        let ts = utc_ts(self.at_midnight().0) + rhs.0;
        Self(ts.to_zoned_datetime(JiffTimeZone::UTC).date())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NaiveTime(civil::Time);

impl NaiveTime {
    pub fn from_hms_opt(h: u32, m: u32, s: u32) -> Option<Self> {
        if h > 23 || m > 59 || s > 59 {
            return None;
        }
        civil::Time::new(h as i8, m as i8, s as i8, 0).ok().map(Self)
    }

    fn on(self, date: NaiveDate) -> NaiveDateTime {
        NaiveDateTime(date.0.at(self.0.hour(), self.0.minute(), self.0.second(), self.0.subsec_nanosecond()))
    }
}

/// A signed duration, with the same constructors MDM used from `chrono::Duration`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Duration(jiff::SignedDuration);

impl Duration {
    pub fn zero() -> Self {
        Self(jiff::SignedDuration::ZERO)
    }
    pub fn seconds(secs: i64) -> Self {
        Self(jiff::SignedDuration::from_secs(secs))
    }
    pub fn minutes(mins: i64) -> Self {
        Self(jiff::SignedDuration::from_secs(mins.saturating_mul(60)))
    }
    pub fn hours(hours: i64) -> Self {
        Self(jiff::SignedDuration::from_secs(hours.saturating_mul(3600)))
    }
    pub fn days(days: i64) -> Self {
        Self(jiff::SignedDuration::from_secs(days.saturating_mul(86_400)))
    }
    pub fn milliseconds(ms: i64) -> Self {
        Self(jiff::SignedDuration::from_millis(ms))
    }
    pub fn from_std(std: std::time::Duration) -> Result<Self, std::time::Duration> {
        jiff::SignedDuration::try_from(std).map(Self).map_err(|_| std)
    }
    pub fn num_milliseconds(&self) -> i64 {
        self.0.as_millis() as i64
    }
}

pub trait TimeZone {
    fn offset_from_utc_datetime(&self, utc: &NaiveDateTime) -> FixedOffset;
}

impl TimeZone for Utc {
    fn offset_from_utc_datetime(&self, _utc: &NaiveDateTime) -> FixedOffset {
        FixedOffset(0)
    }
}

impl TimeZone for Local {
    fn offset_from_utc_datetime(&self, utc: &NaiveDateTime) -> FixedOffset {
        let ts = utc_ts(utc.0);
        FixedOffset(ts.to_zoned_datetime(JiffTimeZone::system()).offset().seconds())
    }
}

impl TimeZone for FixedOffset {
    fn offset_from_utc_datetime(&self, _utc: &NaiveDateTime) -> FixedOffset {
        *self
    }
}

pub enum LocalResult<T> {
    Single(T),
    None,
}

impl LocalResult<DateTime<Local>> {
    pub fn single(self) -> Option<DateTime<Local>> {
        match self {
            Self::Single(v) => Some(v),
            Self::None => None,
        }
    }

    pub fn earliest(self) -> Option<DateTime<Local>> {
        self.single()
    }
}

impl Local {
    pub fn now() -> DateTime<Local> {
        DateTime::from(Timestamp::now())
    }

    pub fn from_local_datetime(&self, local: &NaiveDateTime) -> LocalResult<DateTime<Local>> {
        match local.0.to_zoned_datetime(JiffTimeZone::system()).map(|z| z.timestamp()) {
            Ok(ts) => LocalResult::Single(DateTime::from(ts)),
            Err(_) => LocalResult::None,
        }
    }
}

impl Utc {
    pub fn now() -> DateTime<Utc> {
        DateTime::from(Timestamp::now())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Weekday(i8);

impl Weekday {
    pub fn num_days_from_sunday(self) -> u32 {
        self.0 as u32
    }
}

pub trait Datelike {
    fn year(&self) -> i32;
    fn month(&self) -> u32;
    fn month0(&self) -> u32;
    fn day(&self) -> u32;
    fn weekday(&self) -> Weekday;
}

pub trait Timelike {
    fn hour(&self) -> u32;
    fn minute(&self) -> u32;
    fn second(&self) -> u32;
}

macro_rules! date_like {
    ($t:ty) => {
        impl Datelike for $t {
            fn year(&self) -> i32 { i32::from(self.0.year()) }
            fn month(&self) -> u32 { self.0.month() as u32 }
            fn month0(&self) -> u32 { self.0.month() as u32 - 1 }
            fn day(&self) -> u32 { self.0.day() as u32 }
            fn weekday(&self) -> Weekday { Weekday(self.0.weekday().to_sunday_zero_offset()) }
        }
    };
}

date_like!(NaiveDate);

impl Datelike for NaiveDateTime {
    fn year(&self) -> i32 { i32::from(self.0.year()) }
    fn month(&self) -> u32 { self.0.month() as u32 }
    fn month0(&self) -> u32 { self.0.month() as u32 - 1 }
    fn day(&self) -> u32 { self.0.day() as u32 }
    fn weekday(&self) -> Weekday { Weekday(self.0.date().weekday().to_sunday_zero_offset()) }
}

impl Timelike for NaiveDateTime {
    fn hour(&self) -> u32 { self.0.hour() as u32 }
    fn minute(&self) -> u32 { self.0.minute() as u32 }
    fn second(&self) -> u32 { self.0.second() as u32 }
}

impl<Tz> Datelike for DateTime<Tz> {
    fn year(&self) -> i32 { self.naive_local().year() }
    fn month(&self) -> u32 { self.naive_local().month() }
    fn month0(&self) -> u32 { self.naive_local().month0() }
    fn day(&self) -> u32 { self.naive_local().day() }
    fn weekday(&self) -> Weekday { self.naive_local().weekday() }
}

impl<Tz> Timelike for DateTime<Tz> {
    fn hour(&self) -> u32 { self.naive_local().hour() }
    fn minute(&self) -> u32 { self.naive_local().minute() }
    fn second(&self) -> u32 { self.naive_local().second() }
}

#[derive(Clone, Copy, Debug)]
pub enum SecondsFormat {
    Secs,
    Millis,
    Micros,
    Nanos,
}

fn utc_ts(dt: civil::DateTime) -> Timestamp {
    dt.to_zoned_datetime(JiffTimeZone::UTC)
        .expect("UTC civil times are always unambiguous")
        .timestamp()
}

enum FormattedValue<'a> {
    Timestamp(&'a Timestamp),
    Civil(&'a civil::DateTime),
    Date(&'a civil::Date),
}

pub struct Formatted<'a> {
    fmt: String,
    value: FormattedValue<'a>,
}

impl<'a, Tz> From<&'a DateTime<Tz>> for FormattedValue<'a> {
    fn from(v: &'a DateTime<Tz>) -> Self { Self::Timestamp(&v.ts) }
}

impl<'a> From<&'a NaiveDateTime> for FormattedValue<'a> {
    fn from(v: &'a NaiveDateTime) -> Self { Self::Civil(&v.0) }
}

impl<'a> From<&'a NaiveDate> for FormattedValue<'a> {
    fn from(v: &'a NaiveDate) -> Self { Self::Date(&v.0) }
}

impl fmt::Display for Formatted<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fmt = self.fmt.replace("%.6f", "%f");
        let zoned;
        let text = match self.value {
            FormattedValue::Timestamp(ts) => {
                zoned = ts.to_zoned_datetime(JiffTimeZone::UTC);
                strtime::format(&fmt, &zoned)
            }
            FormattedValue::Civil(dt) => strtime::format(&fmt, *dt),
            FormattedValue::Date(d) => strtime::format(&fmt, *d),
        };
        match text {
            Ok(s) => f.write_str(&s),
            Err(_) => f.write_str(""),
        }
    }
}

pub fn now_zoned() -> ZonedDateTime {
    Timestamp::now().to_zoned_datetime(JiffTimeZone::system())
}

pub fn system_time_to_timestamp(t: SystemTime) -> Option<Timestamp> {
    Timestamp::try_from(t).ok()
}
