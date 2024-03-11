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
