use crate::config::split_uint::split_unit;
use serde::{de::Visitor, Deserialize};

#[derive(Default, Debug, Clone, Copy)]
pub struct DurationInMillis(pub(crate) std::time::Duration);

impl DurationInMillis {
    pub fn new(ms: u64) -> Self {
        Self(std::time::Duration::from_millis(ms))
    }
}

impl std::ops::Deref for DurationInMillis {
    type Target = std::time::Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct DurationInMillsVisitor;

impl<'de> Visitor<'de> for DurationInMillsVisitor {
    type Value = DurationInMillis;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string value for DurationInMillis, such as `12Kb`, `1024`")
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
            return Ok(DurationInMillis::new(0));
        }

        let items = split_unit(v.as_str());
        let mut amount: u64 = 0;

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
                "" | "ms" => {
                    unit = 1;
                }
                "s" => {
                    unit = 1000;
                }
                "m" => {
                    unit = 60 * 1000;
                }
                "h" => {
                    unit = 60 * 60 * 1000;
                }
                "d" => {
                    unit = 24 * 60 * 60 * 1000;
                }
                "w" => {
                    unit = 7 * 24 * 60 * 60 * 1000;
                }
                _ => {
                    return Err(E::custom(format!(
                        "bad unit, `{}` not in `ms,s,m,h,d,w`",
                        units
                    )));
                }
            }

            amount += num * unit;
        }

        Ok(DurationInMillis::new(amount))
    }
}

impl<'de> Deserialize<'de> for DurationInMillis {
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

    use super::DurationInMillis;

    #[derive(Deserialize, Default, Debug)]
    struct Config {
        timeout: DurationInMillis,
    }

    #[test]
    fn duration_in_mills() {
        let config: Config = toml::from_str(
            r#"
    timeout="52"
    "#,
        )
        .unwrap();
        println!("{:?}", config.timeout);
    }
}
