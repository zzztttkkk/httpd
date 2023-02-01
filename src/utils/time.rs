use chrono;

pub type LocalTime = chrono::DateTime<chrono::Local>;
pub type UtcTime = chrono::DateTime<chrono::Utc>;

pub static DEFAULT_TIME_LAYOUT: &str = "%Y-%m-%d %H:%M:%S.%6f";

pub struct Time();

impl Time {
    pub fn currentstr() -> String {
        Self::now().format(DEFAULT_TIME_LAYOUT).to_string()
    }

    pub fn now() -> LocalTime {
        chrono::Local::now()
    }

    pub fn utc() -> UtcTime {
        chrono::Utc::now()
    }

    pub fn duration<T: chrono::TimeZone>(
        begin: chrono::DateTime<T>,
        end: chrono::DateTime<T>,
    ) -> chrono::Duration {
        begin.signed_duration_since(end)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::Time;

    #[test]
    fn x() {
        println!("{}", Time::currentstr());
        println!("{}", Time::currentstr());
    }
}
