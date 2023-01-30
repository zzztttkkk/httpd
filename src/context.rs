use std::{any::Any, collections::HashMap, fmt, ops::Deref, ops::DerefMut};

use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{request::Request, response::Response};

pub struct RwLockWrapper(usize);

impl RwLockWrapper {
    pub async fn read(&self) -> RwLockReadGuard<()> {
        let v: &RwLock<()> = unsafe { std::mem::transmute(self.0) };
        v.read().await
    }

    pub async fn write(&self) -> RwLockWriteGuard<()> {
        let v: &RwLock<()> = unsafe { std::mem::transmute(self.0) };
        v.write().await
    }
}

macro_rules! impl_for_raw_ptr {
    ($name:ident, $target:tt) => {
        impl Deref for $name {
            type Target = $target;

            fn deref(&self) -> &Self::Target {
                unsafe { std::mem::transmute(self.0) }
            }
        }

        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { std::mem::transmute(self.0) }
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let ptr: *const Self = unsafe { std::mem::transmute(self.0) };
                write!(f, "{:?}", ptr)
            }
        }

        impl $name {
            pub(crate) fn new(v: usize) -> Self {
                Self(v)
            }

            pub fn ptr(&self) -> *const Self {
                unsafe { std::mem::transmute(self.0) }
            }
        }
    };
}

pub struct RequestRawPtr(usize);

impl_for_raw_ptr!(RequestRawPtr, Request);

pub struct ResponseRawPtr(usize);

impl_for_raw_ptr!(ResponseRawPtr, Response);

#[derive(Debug)]
pub struct Context {
    _data: HashMap<String, Box<dyn Any>>,
    _req: usize,
    _resp: usize,

    pub(crate) _sync: RwLock<()>,
    pub(crate) _pre_stop: bool,
    pub(crate) _post_stop: bool,
}

unsafe impl Send for Context {}

unsafe impl Sync for Context {}

pub enum MiddlewareCtrl {
    Pre,
    Post,
}

impl Context {
    pub fn new(req: usize, resp: usize) -> Self {
        Self {
            _data: HashMap::new(),
            _req: req,
            _resp: resp,

            _sync: RwLock::new(()),
            _pre_stop: false,
            _post_stop: false,
        }
    }

    pub fn get<T: Any>(&mut self, k: &str) -> Option<&mut T> {
        match self._data.get_mut(k) {
            Some(v) => v.downcast_mut(),
            None => None,
        }
    }

    pub fn sync(&self) -> RwLockWrapper {
        RwLockWrapper(unsafe { std::mem::transmute(&self._sync) })
    }

    pub fn set(&mut self, k: &str, v: Box<dyn Any>) {
        self._data.insert(k.to_string(), v);
    }

    pub fn request(&self) -> RequestRawPtr {
        RequestRawPtr(unsafe { std::mem::transmute(self._req) })
    }

    pub fn response(&self) -> ResponseRawPtr {
        ResponseRawPtr(unsafe { std::mem::transmute(self._resp) })
    }

    pub async fn stop(&mut self, ctrl: MiddlewareCtrl) {
        let sync = self.sync();
        let _r = sync.write().await;
        match ctrl {
            MiddlewareCtrl::Pre => {
                self._pre_stop = true;
            }
            MiddlewareCtrl::Post => {
                self._post_stop = true;
            }
        }
    }
}
