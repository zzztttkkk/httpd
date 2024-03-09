use tokio::io::AsyncWriteExt;
use utils::anyhow;

use crate::{item::Item, Appender, Renderer};

pub(crate) struct Consumer {
    pub(crate) appenders: Vec<Box<dyn Appender>>,
    pub(crate) renderers: Vec<Box<dyn Renderer>>,
    pub(crate) map: Vec<usize>,
}

impl Consumer {
    pub(crate) fn init(&mut self) -> anyhow::Result<()> {
        if self.renderers.is_empty() {
            return anyhow::error("empty renderers");
        }

        if self.appenders.is_empty() {
            return anyhow::error("empty appenders");
        }

        for _ in 0..self.appenders.len() {
            self.map.push(0);
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

    pub(crate) async fn consume_when_single_renderer(
        &mut self,
        item: &Item,
        render_bufs: &mut Vec<Vec<u8>>,
    ) {
        let mut appenders: smallvec::SmallVec<[&mut Box<dyn Appender>; 12]> = smallvec::smallvec![];
        for appender in self.appenders.iter_mut() {
            if appender.filter(item) {
                appenders.push(appender);
            }
        }

        if appenders.len() < 1 {
            return;
        }

        let renderer = unsafe { self.renderers.get_unchecked(0) };
        let buf = unsafe { render_bufs.get_unchecked_mut(0) };
        buf.clear();
        renderer.render(item, buf);

        let mut fs = vec![];
        for appender in appenders.iter_mut() {
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

    pub(crate) async fn comsume(&mut self, item: &Item, render_bufs: &mut Vec<Vec<u8>>) {
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

        if appenders.is_empty() {
            return;
        }

        for ridx in ridxes {
            let buf = unsafe { render_bufs.get_unchecked_mut(ridx) };
            let renderer = unsafe { self.renderers.get_unchecked(ridx) };
            buf.clear();
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

    pub(crate) async fn flush(&mut self) {
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
