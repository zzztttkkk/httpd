use crate::{appender::Renderer, item::Item, Appender};

enum Message {
    Flush(std::sync::Arc<std::sync::Mutex<()>>),
    LogItem(Item),
}

pub struct Dispatcher {
    sx: std::sync::mpsc::Sender<Message>,
}

impl log::Log for Dispatcher {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        self.sx
            .send(Message::LogItem(Item::from(record)))
            .expect("logging: send log message failed");
    }

    fn flush(&self) {
        let lock = std::sync::Arc::new(std::sync::Mutex::new(()));
        self.sx
            .send(Message::Flush(lock.clone()))
            .expect("logging: send flush message failed");

        loop {
            match lock.try_lock() {
                Ok(g) => {
                    std::mem::drop(g);
                    std::thread::sleep(std::time::Duration::from_millis(3));
                }
                Err(_) => {
                    break;
                }
            }
        }

        _ = lock.lock();
    }
}

pub struct Consumer {
    appenders: Vec<Box<dyn Appender>>,
    renderers: Vec<Box<dyn Renderer>>,
}

impl Consumer {
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn comsume(&mut self, msg: &Message) {}
}

pub fn init(
    appenders: Vec<Box<dyn Appender>>,
    renderers: Vec<Box<dyn Renderer>>,
) -> Result<(), log::SetLoggerError> {
    let (sx, rx) = std::sync::mpsc::channel();
    let dispatcher = Dispatcher { sx };
    let ptr = Box::new(dispatcher);

    let mut consumer = Consumer {
        appenders,
        renderers,
    };
    consumer.init();

    std::thread::spawn(move || {
        let comsumer = &mut consumer;
        for msg in rx {
            comsumer.comsume(&msg);
        }
    });

    log::set_max_level(log::Level::Trace.to_level_filter());
    log::set_logger(Box::leak(ptr))
}
