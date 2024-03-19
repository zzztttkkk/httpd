use std::collections::HashMap;

use utils::anyhow;

use crate::{appender::Appender, appender::Renderer, consumer::Consumer, item::Item};

enum Message {
    Flush(std::sync::Arc<std::sync::atomic::AtomicBool>),
    LogItem(Item),
}

struct Dispatcher {
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
        let lock = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.sx
            .send(Message::Flush(lock.clone()))
            .expect("logging: send flush message failed");

        loop {
            if lock.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}

pub struct ShutdownGuard {
    ptr: &'static Dispatcher,
    signal: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        let ptr: *mut Dispatcher = unsafe { std::mem::transmute(self.ptr) };
        let ptr = unsafe { Box::from_raw(ptr) };
        std::mem::drop(ptr);

        loop {
            if self.signal.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
}

pub fn init(
    level: log::Level,
    appenders: Vec<Box<dyn Appender>>,
    renderers: Vec<Box<dyn Renderer>>,
) -> anyhow::Result<ShutdownGuard> {
    let (sx, rx) = std::sync::mpsc::channel();
    let dispatcher = Dispatcher { sx };
    let ptr: &'static Dispatcher = Box::leak(Box::new(dispatcher));

    let guard = ShutdownGuard {
        signal: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        ptr,
    };

    let shutdown_signal = guard.signal.clone();

    let c = appenders.len();
    let mut consumer = Consumer {
        appenders,
        renderers,
        armap: Vec::with_capacity(c),
        servicemap: HashMap::new(),
    };
    consumer.init()?;

    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .max_blocking_threads(consumer.appenders.len())
            .build()
            .unwrap();

        runtime.block_on(async move {
            let mut render_bufs = Vec::with_capacity(consumer.renderers.len());
            for _ in 0..consumer.renderers.len() {
                render_bufs.push(Vec::<u8>::default());
            }
            let consumer = tokio::sync::Mutex::new(consumer);

            macro_rules! consume {
                ($rx:ident, $render_bufs:ident, $consumer:ident, $method:ident) => {
                    for msg in $rx {
                        match msg {
                            Message::Flush(lock) => {
                                let mut consumer = $consumer.lock().await;
                                consumer.flush().await;
                                lock.store(true, std::sync::atomic::Ordering::SeqCst);
                            }
                            Message::LogItem(ref item) => {
                                let mut consumer = $consumer.lock().await;
                                consumer.$method(item, &mut $render_bufs).await;
                            }
                        }
                    }
                };
            }

            if render_bufs.len() == 1 {
                consume!(rx, render_bufs, consumer, consume_when_single_renderer);
            } else {
                consume!(rx, render_bufs, consumer, comsume);
            }
            shutdown_signal.store(true, std::sync::atomic::Ordering::SeqCst);

            let mut consumer = consumer.lock().await;
            consumer.flush().await;
        });
    });

    log::set_max_level(level.to_level_filter());
    anyhow::result(log::set_logger(ptr))?;
    Ok(guard)
}
