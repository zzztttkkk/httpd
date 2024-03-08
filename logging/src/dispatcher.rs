pub struct Dispatcher {}

impl Dispatcher {
    pub fn init(self) -> Result<(), log::SetLoggerError> {
        log::set_max_level(log::Level::Trace.to_level_filter());
        log::set_boxed_logger(Box::new(self))
    }
}

impl log::Log for Dispatcher {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        println!(
            "{} {} {}",
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            record.metadata().level().as_str(),
            record.args()
        )
    }

    fn flush(&self) {}
}
