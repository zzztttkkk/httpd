use serde::{de::Visitor, Deserialize};
use std::fmt;

use super::split_unit::split_unit;

#[derive(Default, Debug, Clone, Copy)]
pub struct SizeInBytes(u64);

impl SizeInBytes {
    #[inline(always)]
    pub fn u64(&self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub fn usize(&self) -> usize {
        self.0 as usize
    }

    #[inline(always)]
    pub fn update(&mut self, v: u64) {
        self.0 = v;
    }

    #[inline(always)]
    pub fn less_then(&mut self, cmp: u64, v: u64) {
        if self.0 < cmp {
            self.0 = v;
        }
    }
}

pub struct SizeInBytesVisitor;

impl<'de> Visitor<'de> for SizeInBytesVisitor {
    type Value = SizeInBytes;

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
        if (v.is_empty()) {
            return Ok(SizeInBytes(0));
        }

        let (nums, units) = split_unit(v.as_str());
        let mut num: u64 = 0;
        match nums.parse::<u64>() {
            Ok(v) => {
                num = v;
            }
            Err(_) => {
                return Err(E::custom(format!("bad number value, `{}`", nums)));
            }
        }

        let mut unit = 1;
        match units.to_lowercase().trim() {
            "k" | "kb" => {
                unit = 1024;
            }
            "m" | "mb" => {
                unit = 1024 * 1024;
            }
            "g" | "gb" => {
                unit = 1204 * 1024 * 1024;
            }
            "" | "b" | "bytes" => {
                unit = 1;
            }
            _ => {
                return Err(E::custom(format!("bad unit, `{}`", units)));
            }
        }
        Ok(SizeInBytes(num * unit))
    }
}

impl<'de> Deserialize<'de> for SizeInBytes {
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

    use super::SizeInBytes;

    #[derive(Deserialize, Default, Debug)]
    struct Config {
        max_size: SizeInBytes,
    }

    #[test]
    fn size_in_bytes() {
        let config: Config = toml::from_str(
            r#"
max_size="12kb"
"#,
        )
        .unwrap();
        println!("{:?}", config);
    }
}
