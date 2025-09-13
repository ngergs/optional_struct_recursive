use crate::optionable::impl_optional_self;
use crate::Optionable;
use chrono::{DateTime, Days, Months, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone};

impl<Tz: TimeZone> Optionable for DateTime<Tz> {
    type Optioned = Self;
}

impl_optional_self!(Days, Months, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta);
