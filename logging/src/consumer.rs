use std::collections::{HashMap, HashSet};

use utils::anyhow;

use crate::{
    appender::{Appender, Renderer},
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

// TODO explain this raw ptr
struct AppenderPtr(*mut Box<dyn Appender>);

impl AppenderPtr {
    fn from(ptr: &mut Box<dyn Appender>) -> Self {
        Self(ptr)
    }

    fn to(&self) -> &'static mut Box<dyn Appender> {
        unsafe { &mut *(self.0) }
    }
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

    fn filter_when_single_renderer(&self, item: &Item, dest: &mut smallvec::SmallVec<[usize; 12]>) {
        match self.servicemap.get(item.service.as_str()) {
            None => {
                return;
            }
            Some(idxes) => {
                for idx in idxes {
                    let appender = unsafe { self.appenders.get_unchecked(*idx) };
                    if appender.filter(item) {
                        dest.push(*idx);
                    }
                }
            }
        }
    }

    pub(crate) async fn consume_when_single_renderer(
        &mut self,
        item: &Item,
        render_bufs: &mut Vec<Vec<u8>>,
    ) {
        let mut appender_idxes: smallvec::SmallVec<[usize; 12]> = smallvec::smallvec![];
        self.filter_when_single_renderer(item, &mut appender_idxes);
        if appender_idxes.len() < 1 {
            return;
        }

        let renderer = unsafe { self.renderers.get_unchecked(0) };
        let buf = unsafe { render_bufs.get_unchecked_mut(0) };
        buf.clear();
        renderer.render(item, buf);

        let mut futs = vec![];
        for idx in appender_idxes {
            let ptr = AppenderPtr::from(unsafe { self.appenders.get_unchecked_mut(idx) });
            futs.push(ptr.to().writeall(&buf));
        }

        for f in futs {
            match f.await {
                Err(e) => {
                    eprintln!("logging: write failed, {}", e);
                }
                _ => {}
            }
        }
    }

    fn filter(
        &self,
        item: &Item,
        dest: &mut smallvec::SmallVec<[usize; 12]>,
        renderer_idxes: &mut smallvec::SmallVec<[usize; 12]>,
        appender_renderer_map: &mut smallvec::SmallVec<[usize; 12]>,
    ) {
        match self.servicemap.get(item.service.as_str()) {
            None => {
                return;
            }
            Some(idxes) => {
                for idx in idxes {
                    let idx = *idx;
                    let appender = unsafe { self.appenders.get_unchecked(idx) };
                    if !appender.filter(item) {
                        continue;
                    }

                    dest.push(idx);
                    let ridx = *unsafe { self.armap.get_unchecked(idx) };
                    appender_renderer_map.push(ridx);
                    if renderer_idxes.contains(&ridx) {
                        continue;
                    }
                    renderer_idxes.push(ridx);
                }
            }
        }
    }

    pub(crate) async fn comsume(&mut self, item: &Item, render_bufs: &mut Vec<Vec<u8>>) {
        let mut using_renderer_idxes: smallvec::SmallVec<[usize; 12]> = smallvec::smallvec![];
        let mut appender_renderer_idx_map: smallvec::SmallVec<[usize; 12]> = smallvec::smallvec![];
        let mut appender_idxes: smallvec::SmallVec<[usize; 12]> = smallvec::smallvec![];
        self.filter(
            item,
            &mut appender_idxes,
            &mut using_renderer_idxes,
            &mut appender_renderer_idx_map,
        );
        if appender_idxes.is_empty() {
            return;
        }

        for ridx in using_renderer_idxes {
            let buf = unsafe { render_bufs.get_unchecked_mut(ridx) };
            let renderer = unsafe { self.renderers.get_unchecked(ridx) };
            buf.clear();
            renderer.render(item, buf);
        }

        let mut futs: smallvec::SmallVec<[_; 12]> = smallvec::smallvec![];
        for idx in appender_idxes {
            let buf = unsafe {
                render_bufs.get_unchecked(*(appender_renderer_idx_map.get_unchecked(idx)))
            };
            let ptr = AppenderPtr::from(unsafe { self.appenders.get_unchecked_mut(idx) });
            futs.push(ptr.to().writeall(&buf));
        }

        for f in futs {
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
