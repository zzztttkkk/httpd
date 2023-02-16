use std::time::Duration;

use serde::{de::Visitor, Deserialize};

use super::split_unit::split_unit;

#[derive(Default, Debug, Clone, Copy)]
pub struct DurationInMills(u64);

impl DurationInMills {
    #[inline(always)]
    pub fn u64(&self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub fn duration(&self) -> Duration {
        Duration::from_millis(self.0)
    }

    #[inline(always)]
    pub fn less_then(&mut self, cmp: u64, v: u64) {
        if self.0 < cmp {
            self.0 = v;
        }
    }
}

pub struct DurationInMillsVisitor;

impl<'de> Visitor<'de> for DurationInMillsVisitor {
    type Value = DurationInMills;

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
            return Ok(DurationInMills(0));
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
            "" | "ms" | "mills" => {
                unit = 1;
            }
            "s" | "sec" | "seconds" => {
                unit = 1000;
            }
            "m" | "min" | "minutes" | "minute" => {
                unit = 60 * 1000;
            }
            "h" | "hours" | "hour" => {
                unit = 60 * 60 * 1000;
            }
            "d" | "days" | "day" => {
                unit = 24 * 60 * 60 * 1000;
            }
            _ => {
                return Err(E::custom(format!("bad unit, `{}`", units)));
            }
        }
        Ok(DurationInMills(num * unit))
    }
}

impl<'de> Deserialize<'de> for DurationInMills {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(DurationInMillsVisitor)
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::DurationInMills;

    #[derive(Deserialize, Default, Debug)]
    struct Config {
        timeout: DurationInMills,
    }

    #[test]
    fn duration_in_mills() {
        let config: Config = toml::from_str(
            r#"
timeout="1d"
"#,
        )
        .unwrap();
        println!("{:?}", config);
    }
}
