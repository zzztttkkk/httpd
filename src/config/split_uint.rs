use std::cell::RefCell;

pub(crate) fn split_unit(v: &str) -> Vec<(String, String)> {
    let mut items: Vec<(String, String)> = vec![("".to_string(), "".to_string())];

    let mut prev = None;
    for char in v.chars() {
        if char.is_whitespace() {
            continue;
        }

        if prev.is_none() {
            prev = Some(char);
            continue;
        }

        if char.is_numeric() {
            items.last_mut().unwrap().0.push(char);
        } else {
            items.last_mut().unwrap().1.push(char);
        }
    }
    return items;
}

#[cfg(test)]
mod tests {
    use super::split_unit;

    #[test]
    fn test_split_unit() {
        println!("{:?}", split_unit("1d2h"))
    }
}
