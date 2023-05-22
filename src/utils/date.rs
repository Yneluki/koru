use chrono::{DateTime, Timelike, Utc};

/// Builds an UTC now for now, without nano-seconds.
pub fn now() -> DateTime<Utc> {
    let date = Utc::now();
    date.with_nanosecond(date.nanosecond() - (date.nanosecond() % 1000))
        .expect("now to be able to have 0 nanoseconds")
}
