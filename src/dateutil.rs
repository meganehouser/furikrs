use chrono::{Local, DateTime, NaiveDateTime, Utc, TimeZone};
use failure::{Error, format_err};

pub fn naive_str_to_utc(date: &str, time: &str) -> Result<DateTime<Utc>, Error> {
    let datetime = [date, time].join(" ");
    let naive = NaiveDateTime::parse_from_str(&datetime, "%Y-%m-%d %H:%M:%S%.f")?;
    let local = Local.from_local_datetime(&naive).single()
        .ok_or_else(|| format_err!("can't parse datetime '{}'", date))?;
    let utc = local.with_timezone(&Utc);
    Ok(utc)
}

