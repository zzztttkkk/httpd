use std::cell::RefCell;

pub(crate) fn split_unit(v: &str) -> Vec<(String, String)> {
    let mut items: Vec<(String, String)> = vec![Default::default()];

    let mut prev = None;
    let mut unit_begin = false;
    for char in v.chars() {
        if char.is_whitespace() {
            continue;
        }

        if char.is_numeric() {
            if unit_begin {
                items.push(Default::default());
                unit_begin = false;
            }
            items.last_mut().unwrap().0.push(char);
        } else {
            if items.last().unwrap().0.is_empty() {
                panic!("httpd.config: bad unit string, `{}`", v);
            }

            items.last_mut().unwrap().1.push(char);
        }

        if prev.is_none() {
            prev = Some(char);
        } else {
            if (prev.unwrap().is_numeric() && !char.is_numeric()) {
                unit_begin = true;
            }
        }

        prev = Some(char);
    }
    return items;
}

#[cfg(test)]
mod tests {
    use super::split_unit;

    #[test]
    fn test_split_unit() {
        let items = split_unit("1days2hours3");
        println!("{:?}", items);
    }
}
