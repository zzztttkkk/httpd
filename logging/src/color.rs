use log::Level;

#[derive(Default, Clone, Copy, Debug)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub(crate) fn is_block(&self) -> bool {
        self.0 == 0 && self.1 == 0 && self.2 == 1
    }
}

#[derive(Debug)]
pub struct LevelColors {
    pub trace: Option<Color>,
    pub debug: Option<Color>,
    pub info: Option<Color>,
    pub warn: Option<Color>,
    pub error: Option<Color>,
}

impl LevelColors {
    pub fn get(&self, level: Level) -> Option<Color> {
        match level {
            Level::Error => self.error,
            Level::Warn => self.warn,
            Level::Info => self.info,
            Level::Debug => self.debug,
            Level::Trace => self.trace,
        }
    }
}

impl Default for LevelColors {
    fn default() -> Self {
        Self {
            trace: Some(Color(94, 102, 91)),
            debug: Some(Color(46, 49, 124)),
            info: Some(Color(91, 174, 35)),
            warn: Some(Color(252, 211, 55)),
            error: Some(Color(237, 90, 101)),
        }
    }
}

#[derive(Debug)]
pub struct ColorScheme {
    pub levels: Option<LevelColors>,
    pub time: Option<Color>,

    pub file: Option<Color>,
    pub line: Option<Color>,

    pub key: Option<Color>,
    pub value: Option<Color>,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            levels: Some(Default::default()),
            time: Some(Color(106, 64, 40)),
            file: Some(Color(85, 187, 138)),
            line: Some(Color(20, 145, 168)),
            key: Some(Color(216, 89, 22)),
            value: Some(Color(20, 30, 27)),
        }
    }
}
