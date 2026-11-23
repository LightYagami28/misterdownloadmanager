//! Time zones, offsets and calendar traits.

use jiff::{Timestamp, tz::TimeZone as JiffTimeZone};

use crate::{DateTime, NaiveDate, NaiveDateTime, utc_ts};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Utc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Local;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedOffset(pub(crate) i32);

impl FixedOffset {
    pub fn east_opt(secs: i32) -> Option<Self> {
        if secs < -86_399 || secs > 86_399 {
            None
        } else {
            Some(Self(secs))
        }
    }

    pub fn local_minus_utc(self) -> i32 {
        self.0
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

macro_rules! civil_datelike {
    ($t:ty, $date:expr) => {
        impl Datelike for $t {
            fn year(&self) -> i32 { i32::from($date(self).year()) }
            fn month(&self) -> u32 { $date(self).month() as u32 }
            fn month0(&self) -> u32 { $date(self).month() as u32 - 1 }
            fn day(&self) -> u32 { $date(self).day() as u32 }
            fn weekday(&self) -> Weekday {
                Weekday($date(self).weekday().to_sunday_zero_offset())
            }
        }
    };
}

civil_datelike!(NaiveDate, |v: &NaiveDate| v.0);
civil_datelike!(NaiveDateTime, |v: &NaiveDateTime| v.0.date());

impl<Tz> Datelike for DateTime<Tz> {
    fn year(&self) -> i32 { self.naive_local().year() }
    fn month(&self) -> u32 { self.naive_local().month() }
    fn month0(&self) -> u32 { self.naive_local().month0() }
    fn day(&self) -> u32 { self.naive_local().day() }
    fn weekday(&self) -> Weekday { self.naive_local().weekday() }
}

impl Timelike for NaiveDateTime {
    fn hour(&self) -> u32 { self.0.hour() as u32 }
    fn minute(&self) -> u32 { self.0.minute() as u32 }
    fn second(&self) -> u32 { self.0.second() as u32 }
}

impl<Tz> Timelike for DateTime<Tz> {
    fn hour(&self) -> u32 { self.naive_local().hour() }
    fn minute(&self) -> u32 { self.naive_local().minute() }
    fn second(&self) -> u32 { self.naive_local().second() }
}
