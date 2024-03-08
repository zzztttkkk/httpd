use std::fmt::format;

use utils::anyhow;

use crate::{
    appender::{self, Renderer},
    item::Item,
    Appender,
};

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
    map: Vec<usize>,
}

impl Consumer {
    fn init(&mut self) -> anyhow::Result<()> {
        if self.renderers.is_empty() {
            return anyhow::error("empty renderers");
        }

        if self.appenders.is_empty() {
            return anyhow::error("empty appenders");
        }

        let mut unused_idxes = vec![];
        'outer: for (ri, rref) in self.renderers.iter().enumerate() {
            for aref in self.appenders.iter() {
                if rref.name() == aref.renderer() {
                    break 'outer;
                }
                unused_idxes.push(ri);
            }
        }

        self.appenders.clear();

        'outer: for (ai, appender) in self.appenders.iter().enumerate() {
            for (ri, renderer) in self.renderers.iter().enumerate() {
                if appender.renderer() == renderer.name() {
                    self.map[ai] = ri;
                    break 'outer;
                }
            }
            return anyhow::error(&format!("renderer `{}` not found", appender.renderer()));
        }
        Ok(())
    }

    fn comsume(&mut self, item: &Item) {
        let appenders = self
            .appenders
            .iter_mut()
            .enumerate()
            .filter(|(_, aref)| aref.filter(item));

        let fc = self.renderers.len();
        for x in appenders {}
    }

    fn flush(&mut self) {}
}

enum RAIdx {
    RIdx(usize),
    AIdx(usize),
}

pub fn init(
    appenders: Vec<Box<dyn Appender>>,
    renderers: Vec<Box<dyn Renderer>>,
) -> anyhow::Result<()> {
    let (sx, rx) = std::sync::mpsc::channel();
    let dispatcher = Dispatcher { sx };
    let ptr = Box::new(dispatcher);

    let c = appenders.len();
    let mut consumer = Consumer {
        appenders,
        renderers,
        map: Vec::with_capacity(c),
    };
    consumer.init()?;

    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .max_blocking_threads(1)
            .build()
            .unwrap()
            .block_on(async move {
                let comsumer = &mut consumer;

                let mut idexes: Vec<RAIdx> = Vec::with_capacity(
                    std::cmp::max(comsumer.appenders.len(), comsumer.renderers.len()) * 3,
                );

                for msg in rx {
                    match &msg {
                        Message::Flush(lock) => {
                            let _g = lock.lock().expect("acquire flush lock failed");
                            comsumer.flush();
                        }
                        Message::LogItem(item) => {
                            comsumer.comsume(item);
                        }
                    }
                }
            })
    });

    log::set_max_level(log::Level::Trace.to_level_filter());
    anyhow::result(log::set_logger(Box::leak(ptr)))
}
