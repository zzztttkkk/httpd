use chrono::{Datelike, TimeZone, Timelike};

use crate::anyhow;

#[inline]
pub fn unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[inline]
pub fn unixmills() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[inline]
pub fn unixnanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

#[inline]
pub fn fmtlocal(v: std::time::SystemTime, fmt: &str) -> String {
    let v: chrono::DateTime<chrono::Local> = v.into();
    v.format(fmt).to_string()
}

#[inline]
pub fn nowlocal() -> chrono::DateTime<chrono::Local> {
    chrono::Local::now()
}

pub fn local(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
    milli: u32,
) -> anyhow::Result<chrono::DateTime<chrono::Local>> {
    let nv = anyhow::option(
        chrono::NaiveDate::from_ymd_opt(year, month, day),
        "to local datetime failed, date",
    )?;
    let ndv = anyhow::option(
        nv.and_hms_milli_opt(hour, min, sec, milli),
        "to local datetime failed, time",
    )?;

    match chrono::Local.from_local_datetime(&ndv) {
        chrono::LocalResult::None => anyhow::error("to local datetime failed, local"),
        chrono::LocalResult::Single(v) => Ok(v),
        chrono::LocalResult::Ambiguous(_, _) => anyhow::error("to local datetime failed, range"),
    }
}

pub fn endofday(dt: Option<std::time::SystemTime>) -> anyhow::Result<u128> {
    let lv: chrono::DateTime<chrono::Local> = match dt {
        Some(dt) => dt.into(),
        None => std::time::SystemTime::now().into(),
    };
    let elv = local(lv.year(), lv.month(), lv.day(), 0, 0, 0, 0)?
        + chrono::Duration::try_days(1).unwrap();
    let esv: std::time::SystemTime = elv.into();
    return Ok(esv
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());
}

pub fn endofhour(dt: Option<std::time::SystemTime>) -> anyhow::Result<u128> {
    let lv: chrono::DateTime<chrono::Local> = match dt {
        Some(dt) => dt.into(),
        None => std::time::SystemTime::now().into(),
    };
    let elv = local(lv.year(), lv.month(), lv.day(), lv.hour(), 0, 0, 0)?
        + chrono::Duration::try_hours(1).unwrap();
    let esv: std::time::SystemTime = elv.into();
    return Ok(esv
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());
}

#[cfg(test)]
mod tests {
    use crate::luxon::{endofday, endofhour, local};

    #[test]
    fn test_local() {
        println!("{:?}", local(12, 12, 3, 23, 14, 45, 566));

        println!("{:?} {:?}", endofhour(None), endofday(None));

        println!("{}", chrono::Local::now().to_rfc2822());
    }
}
