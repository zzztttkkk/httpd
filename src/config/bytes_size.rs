use serde::{de::Visitor, Deserialize};

use super::split_uint::split_unit;

#[derive(Debug, Default, Clone, Copy)]
pub struct BytesSize(pub(crate) usize);

pub struct SizeInBytesVisitor;

impl<'de> Visitor<'de> for SizeInBytesVisitor {
    type Value = BytesSize;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string value for SizeInBytes, such as `12Kb`, `1024`")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_string(v.to_string())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.is_empty() {
            return Ok(BytesSize(0));
        }

        let items = split_unit(v.as_str());
        let mut amount: usize = 0;

        for (nums, units) in items {
            let num: u64;
            match nums.parse::<u64>() {
                Ok(v) => {
                    num = v;
                }
                Err(_) => {
                    return Err(E::custom(format!("bad number value, `{}`", nums)));
                }
            }

            let unit;
            match units.to_lowercase().trim() {
                "" | "b" => {
                    unit = 1;
                }
                "k" | "kb" => {
                    unit = 1024;
                }
                "m" | "mb" => {
                    unit = 1024 * 1024;
                }
                "g" | "gb" => {
                    unit = 1204 * 1024 * 1024;
                }
                _ => {
                    return Err(E::custom(format!("bad unit, `{}` not in `b,k,m,g`", units)));
                }
            }

            amount += (num * unit) as usize;
        }

        Ok(BytesSize(amount))
    }
}

impl<'de> Deserialize<'de> for BytesSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(SizeInBytesVisitor)
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::BytesSize;

    #[derive(Deserialize, Default, Debug)]
    struct Config {
        max_size: BytesSize,
    }

    #[test]
    fn size_in_bytes() {
        let config: Config = toml::from_str(
            r#"
max_size="1kb"
"#,
        )
        .unwrap();
        println!("{:?}", config.max_size);
    }
}
