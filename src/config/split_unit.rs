static DECIMAL_CHARS: &'static str = "0123456789";

pub(crate) fn split_unit<'a>(v: &'a str) -> (&'a str, &'a str) {
    let mut unit_idx = -1;
    for (idx, c) in v.as_bytes().iter().enumerate() {
        if (!DECIMAL_CHARS.contains(*c as char)) {
            unit_idx = idx as i32;
            break;
        }
    }
    if (unit_idx < 0) {
        return (v, "");
    }
    return (&(v[0..(unit_idx as usize)]), &(v[unit_idx as usize..]));
}

#[cfg(test)]
mod tests {
    use super::split_unit;

    #[test]
    fn test_split_unit() {
        assert_eq!(split_unit("12"), ("12", ""));
        assert_eq!(split_unit(""), ("", ""));
        assert_eq!(split_unit("12m"), ("12", "m"));
        assert_eq!(split_unit("1 2,m"), ("1", " 2,m"));
    }
}
