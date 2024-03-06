use std::time::SystemTime;

use crate::level::Level;

pub struct SourceInfo {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

pub struct Event {
    pub time: SystemTime,
    pub source: Option<SourceInfo>,
    pub level: Level,
    pub msg: String,
    pub pairs: Option<Vec<(String, String)>>,
}
