//! A signed duration with the constructors MDM used from `chrono::Duration`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Duration(jiff::SignedDuration);

impl Duration {
    pub(crate) fn new(inner: jiff::SignedDuration) -> Self {
        Self(inner)
    }

    pub(crate) fn get(self) -> jiff::SignedDuration {
        self.0
    }

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
