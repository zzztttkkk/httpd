use std::time::{SystemTime, UNIX_EPOCH};

use once_cell::sync::Lazy;

pub type LocalTime = chrono::DateTime<chrono::Local>;
pub type UtcTime = chrono::DateTime<chrono::Utc>;

pub static DEFAULT_TIME_LAYOUT: &str = "%Y-%m-%d %H:%M:%S.%6f";
pub static DEFAULT_HTTP_HEADER_TIME_LAYOUT: &str = "%a, %d %b %Y %H:%M:%S GMT";
pub static RFC_850: &str = "";
pub static RFC_ANSIC: &str = "";

static HTTP_HEADER_TIME_LAYOUTS: Lazy<Vec<&str>> =
    Lazy::new(|| vec![DEFAULT_HTTP_HEADER_TIME_LAYOUT, RFC_850, RFC_ANSIC]);

#[inline]
pub fn currentstr(layout: Option<&str>) -> String {
    match layout.as_ref() {
        None => {
            now().format(DEFAULT_TIME_LAYOUT).to_string()
        }
        Some(layout) => {
            now().format(*layout).to_string()
        }
    }
}

#[inline]
pub fn now() -> LocalTime {
    chrono::Local::now()
}

#[inline]
pub fn utc() -> UtcTime {
    chrono::Utc::now()
}

#[inline]
pub fn duration<T: chrono::TimeZone>(
    begin: chrono::DateTime<T>,
    end: chrono::DateTime<T>,
) -> chrono::Duration {
    begin.signed_duration_since(end)
}

#[inline]
pub fn utc_from(st: &SystemTime) -> UtcTime {
    let now = st.duration_since(UNIX_EPOCH).unwrap();
    let naive = chrono::NaiveDateTime::from_timestamp_opt(
        now.as_secs() as i64,
        now.subsec_nanos() as u32,
    )
        .unwrap();
    chrono::DateTime::from_utc(naive, chrono::Utc)
}

pub fn parse_from_header_value(txt: &str) -> Option<UtcTime> {
    for layout in HTTP_HEADER_TIME_LAYOUTS.iter() {
        match chrono::DateTime::parse_from_str(txt, &layout) {
            Err(_) => {}
            Ok(t) => {
                return Some(chrono::DateTime::from(t));
            }
        };
    }
    return None;
}


