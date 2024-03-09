use tokio::io::AsyncWriteExt;
use utils::anyhow;

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

        let mut unused_renderer_idxes = vec![];
        'outer: for (ri, rref) in self.renderers.iter().enumerate() {
            for aref in self.appenders.iter() {
                if rref.name() == aref.renderer() {
                    break 'outer;
                }
                unused_renderer_idxes.push(ri);
            }
        }
        unused_renderer_idxes.iter().for_each(|i| {
            self.renderers.remove(*i);
        });

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

    async fn comsume(&mut self, item: &Item, render_bufs: &mut Vec<Vec<u8>>) {
        let mut ridxes: smallvec::SmallVec<[usize; 12]> = smallvec::smallvec![];
        let mut armap: smallvec::SmallVec<[usize; 12]> = smallvec::smallvec![];
        let mut appenders: smallvec::SmallVec<[&mut Box<dyn Appender>; 12]> = smallvec::smallvec![];
        for (aidx, appender) in self.appenders.iter_mut().enumerate() {
            if !appender.filter(item) {
                continue;
            }
            appenders.push(appender);
            let ridx = *unsafe { self.map.get_unchecked(aidx) };
            armap.push(ridx);

            if ridxes.contains(&ridx) {
                continue;
            }
            ridxes.push(ridx);
        }

        for ridx in ridxes {
            let buf = unsafe { render_bufs.get_unchecked_mut(ridx) };
            let renderer = unsafe { self.renderers.get_unchecked(ridx) };
            renderer.render(item, buf);
        }

        let mut fs = vec![];
        for (idx, appender) in appenders.iter_mut().enumerate() {
            let buf = unsafe { render_bufs.get_unchecked(*(armap.get_unchecked(idx))) };
            fs.push(appender.write_all(&buf));
        }

        for wr in futures::future::join_all(fs).await {
            match wr {
                Err(e) => {
                    eprintln!("logging: write failed, {}", e);
                }
                _ => {}
            }
        }
    }

    async fn flush(&mut self) {
        let mut fs = vec![];
        for appender in self.appenders.iter_mut() {
            fs.push(appender.flush());
        }

        for wr in futures::future::join_all(fs).await {
            match wr {
                Err(e) => {
                    eprintln!("logging: flush failed, {}", e);
                }
                _ => {}
            }
        }
    }
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
                let mut render_bufs = Vec::with_capacity(consumer.renderers.len());
                for _ in 0..consumer.renderers.len() {
                    render_bufs.push(Vec::<u8>::default());
                }

                let consumer = tokio::sync::Mutex::new(consumer);

                for msg in rx {
                    match msg {
                        Message::Flush(lock) => {
                            let _g = lock.lock().unwrap();
                            let mut consumer = consumer.lock().await;
                            consumer.flush().await;
                        }
                        Message::LogItem(ref item) => {
                            let mut consumer = consumer.lock().await;
                            consumer.comsume(item, &mut render_bufs).await;
                        }
                    }
                }
            })
    });

    log::set_max_level(log::Level::Trace.to_level_filter());
    anyhow::result(log::set_logger(Box::leak(ptr)))
}
