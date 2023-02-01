use chrono;

pub type LocalTime = chrono::DateTime<chrono::Local>;
pub type UtcTime = chrono::DateTime<chrono::Utc>;

pub static DEFAULT_TIME_LAYOUT: &str = "%Y-%m-%d %H:%M:%S.%6f";

pub struct Time();

impl Time {
    #[inline]
    pub fn currentstr() -> String {
        Self::now().format(DEFAULT_TIME_LAYOUT).to_string()
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
