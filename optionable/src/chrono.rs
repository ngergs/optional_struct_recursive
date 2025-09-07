use crate::{impl_optional_self, Optionable};
use chrono::{DateTime, Days, Months, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone};

#[cfg(feature = "chrono")]
impl<Tz: TimeZone> Optionable for DateTime<Tz> {
    type Optioned = Self;
}

impl_optional_self!(Days, Months, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta);
