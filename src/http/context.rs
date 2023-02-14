use std::{any::Any, collections::HashMap, fmt, ops::Deref, ops::DerefMut};

use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{http::request::Request, http::response::Response};

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

#[derive(Clone, Copy)]
pub struct RequestRawPtr(usize);

impl_for_raw_ptr!(RequestRawPtr, Request);

#[derive(Clone, Copy)]
pub struct ResponseRawPtr(usize);

impl_for_raw_ptr!(ResponseRawPtr, Response);

#[derive(Debug)]
pub struct Context {
    pub(crate) _data: HashMap<String, Box<dyn Any>>,
    pub(crate) _req: RequestRawPtr,
    pub(crate) _resp: ResponseRawPtr,

    pub(crate) _sync: RwLock<()>,
    pub(crate) _pre_stop: bool,
    pub(crate) _post_stop: bool,
    pub(crate) _upgrade_to: Option<String>,
}

unsafe impl Send for Context {}

unsafe impl Sync for Context {}

pub enum MiddlewareCtrl {
    Pre,
    Post,
}

impl Context {
    pub fn new(req: &Box<Request>, resp: &Box<Response>) -> Self {
        Self {
            _data: HashMap::new(),
            _req: unsafe { std::mem::transmute(req) },
            _resp: unsafe { std::mem::transmute(resp) },

            _sync: RwLock::new(()),
            _pre_stop: false,
            _post_stop: false,
            _upgrade_to: None,
        }
    }

    pub fn sync(&self) -> RwLockWrapper {
        RwLockWrapper(unsafe { std::mem::transmute(&self._sync) })
    }

    pub fn get<T: Any>(&mut self, k: &str) -> Option<&mut T> {
        match self._data.get_mut(k) {
            Some(v) => v.downcast_mut(),
            None => None,
        }
    }

    pub fn set(&mut self, k: &str, v: Box<dyn Any>) {
        self._data.insert(k.to_string(), v);
    }

    pub fn request(&mut self) -> RequestRawPtr {
        self._req
    }

    pub fn response(&mut self) -> ResponseRawPtr {
        self._resp
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

    pub fn upgrade(&mut self, proto_name: &str) {
        self._upgrade_to = Some(proto_name.to_ascii_lowercase())
    }
}
