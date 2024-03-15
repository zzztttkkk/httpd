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

pub fn init(
    level: log::Level,
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
        let runtime = tokio::runtime::Builder::new_current_thread()
            .max_blocking_threads(1)
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
        })
    });

    log::set_max_level(level.to_level_filter());
    anyhow::result(log::set_logger(Box::leak(ptr)))
}
