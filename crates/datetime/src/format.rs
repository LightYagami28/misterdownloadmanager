//! strftime-style formatting for the local types.

use std::fmt;

use jiff::{Timestamp, civil, fmt::strtime, tz::TimeZone as JiffTimeZone};

use crate::{DateTime, NaiveDate, NaiveDateTime};

#[derive(Clone, Copy, Debug)]
pub enum SecondsFormat {
    Secs,
    Millis,
    Micros,
    Nanos,
}

pub(crate) enum FormattedValue<'a> {
    Timestamp(&'a Timestamp),
    Civil(&'a civil::DateTime),
    Date(&'a civil::Date),
}

pub struct Formatted<'a> {
    fmt: String,
    value: FormattedValue<'a>,
}

impl<'a, Tz> From<&'a DateTime<Tz>> for FormattedValue<'a> {
    fn from(v: &'a DateTime<Tz>) -> Self {
        Self::Timestamp(&v.ts)
    }
}

impl<'a> From<&'a NaiveDateTime> for FormattedValue<'a> {
    fn from(v: &'a NaiveDateTime) -> Self {
        Self::Civil(&v.0)
    }
}

impl<'a> From<&'a NaiveDate> for FormattedValue<'a> {
    fn from(v: &'a NaiveDate) -> Self {
        Self::Date(&v.0)
    }
}

impl<'a> Formatted<'a> {
    pub(crate) fn new(fmt: &str, value: FormattedValue<'a>) -> Self {
        Self { fmt: fmt.to_owned(), value }
    }
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
