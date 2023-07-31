use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};
use tokio::sync::{Mutex, MutexGuard};
use crate::http::request::Request;
use crate::http::response::Response;

pub struct Context {
    _remote_addr: SocketAddr,
    pub(crate) mutex: Mutex<()>,
    pub request: Request,
    pub response: Response,
}

unsafe impl Send for Context {}

unsafe impl Sync for Context {}

impl Context {
    pub fn new(addr: SocketAddr) -> Self {
        return Self {
            _remote_addr: addr,
            mutex: Mutex::new(()),
            response: Response::new(),
            request: Request::new(),
        };
    }

    pub unsafe fn ptr(&self) -> ContextPtr {
        return ContextPtr(std::mem::transmute(self));
    }

    pub async fn sync(&mut self) -> MutexGuard<()> {
        return self.mutex.lock().await;
    }

    pub fn remote_addr(&self) -> &SocketAddr {
        return &self._remote_addr;
    }
}

// i think Arc<Mutex<Context>> is too expensive.
#[derive(Copy, Clone)]
pub struct ContextPtr(usize);

impl Deref for ContextPtr {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        return unsafe { std::mem::transmute(self.0) };
    }
}

impl DerefMut for ContextPtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return unsafe { std::mem::transmute(self.0) };
    }
}
