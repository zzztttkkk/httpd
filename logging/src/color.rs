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
            trace: Some(Color(0, 170, 144)),
            debug: Some(Color(46, 169, 223)),
            info: Some(Color(165, 222, 228)),
            warn: Some(Color(233, 139, 42)),
            error: Some(Color(203, 27, 69)),
        }
    }
}

#[derive(Debug)]
pub struct ColorScheme {
    pub levels: Option<LevelColors>,
    pub time: Option<Color>,

    pub file: Option<Color>,
    pub line: Option<Color>,

    pub msg: Option<Color>,

    pub key: Option<Color>,
    pub value: Option<Color>,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            levels: Some(Default::default()),
            time: Some(Color(106, 64, 40)),
            file: Some(Color(97, 97, 56)),
            line: Some(Color(93, 172, 129)),
            msg: Some(Default::default()),
            key: Some(Color(37, 83, 89)),
            value: Some(Color(15, 37, 64)),
        }
    }
}
