use std::collections::{HashMap, HashSet};

use utils::anyhow;

use crate::{
    appender::{self, Appender, Renderer},
    item::Item,
};

pub(crate) struct Consumer {
    pub(crate) appenders: Vec<Box<dyn Appender>>,
    pub(crate) renderers: Vec<Box<dyn Renderer>>,
    pub(crate) armap: Vec<usize>, // appender_idx -> renderer_idx
    pub(crate) servicemap: HashMap<String, Vec<usize>>, // service_name -> vec[appender_idx]
}

fn unique(names: impl Iterator<Item = String>) -> bool {
    let mut ns = HashSet::new();
    let mut c = 0;

    for n in names {
        ns.insert(n.to_lowercase());
        c += 1;
    }
    ns.len() == c
}

impl Consumer {
    fn ridx(&self, name: &str) -> Option<usize> {
        self.renderers
            .iter()
            .position(|r| r.name().eq_ignore_ascii_case(name))
    }

    pub(crate) fn init(&mut self) -> anyhow::Result<()> {
        if self.renderers.is_empty() {
            return anyhow::error("empty renderers");
        }

        if self.appenders.is_empty() {
            return anyhow::error("empty appenders");
        }

        if !unique(self.renderers.iter().map(|r| r.name().to_string())) {
            return anyhow::error("repeated renderers");
        }

        for appender in self.appenders.iter() {
            match self.ridx(appender.renderer()) {
                Some(ridx) => {
                    self.armap.push(ridx);
                }
                None => {
                    return anyhow::error(&format!(
                        "renderer `{}` is not found",
                        appender.renderer()
                    ));
                }
            }
        }

        for (idx, appender) in self.appenders.iter().enumerate() {
            match self.servicemap.get_mut(appender.service()) {
                Some(idxes) => {
                    idxes.push(idx);
                }
                None => {
                    self.servicemap
                        .insert(appender.service().to_string(), vec![idx]);
                }
            }
        }
        Ok(())
    }

    pub(crate) async fn consume_when_single_renderer(
        &mut self,
        item: &Item,
        render_bufs: &mut Vec<Vec<u8>>,
    ) {
        let mut appender_idxes: smallvec::SmallVec<[usize; 12]> = smallvec::smallvec![];
        match self.servicemap.get(item.service.as_str()) {
            None => {
                return;
            }
            Some(idxes) => {
                for idx in idxes {
                    let appender = unsafe { self.appenders.get_unchecked(*idx) };
                    if appender.filter(item) {
                        appender_idxes.push(*idx);
                    }
                }
            }
        }

        if appender_idxes.len() < 1 {
            return;
        }

        let renderer = unsafe { self.renderers.get_unchecked(0) };
        let buf = unsafe { render_bufs.get_unchecked_mut(0) };
        buf.clear();
        renderer.render(item, buf);

        let mut futs = vec![];
        for idx in appender_idxes {
            let appender = unsafe { self.appenders.get_unchecked_mut(idx) };
            let fut = appender.writeall(&buf);
            futs.push(fut);
        }
        // for (idx, appender) in self.appenders.iter_mut().enumerate() {
        //     if !appender_idxes.contains(&idx) {
        //         continue;
        //     }
        //     futs.push(appender.writeall(&buf));
        // }

        for f in futs {
            match f.await {
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
            let ridx = *unsafe { self.armap.get_unchecked(aidx) };
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

        let mut fs: smallvec::SmallVec<[_; 12]> = smallvec::smallvec![];
        for (idx, appender) in appenders.iter_mut().enumerate() {
            let buf = unsafe { render_bufs.get_unchecked(*(armap.get_unchecked(idx))) };
            fs.push(appender.writeall(&buf));
        }

        for f in fs {
            match f.await {
                Err(e) => {
                    eprintln!("logging: write failed, {}", e);
                }
                _ => {}
            };
        }
    }

    pub(crate) async fn flush(&mut self) {
        let mut fs: smallvec::SmallVec<[_; 12]> = smallvec::smallvec![];
        for appender in self.appenders.iter_mut() {
            fs.push(appender.flush());
        }

        for f in fs {
            match f.await {
                Err(e) => {
                    eprintln!("logging: flush failed, {}", e);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    struct X {
        num: i32,
    }

    #[test]
    fn test_iter() {
        let mut nums = vec![X { num: 1 }, X { num: 2 }, X { num: 3 }];

        let mut tmp = vec![];
        for idx in vec![0, 2] {
            tmp.push(unsafe { nums.get_unchecked_mut(idx) });
        }

        for v in tmp {
            v.num += 12;
        }
    }
}
