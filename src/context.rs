use std::{any::Any, collections::HashMap};

use crate::{request::Request, response::Response};

#[derive(Debug)]
pub struct Context {
    _data: HashMap<String, Box<dyn Any>>,
    _req: usize,
    _resp: usize,
}

unsafe impl Send for Context {}

unsafe impl Sync for Context {}

impl Context {
    pub fn new(req: usize, resp: usize) -> Self {
        Self {
            _data: HashMap::new(),
            _req: req,
            _resp: resp,
        }
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

    pub fn request(&mut self) -> &mut Request {
        unsafe { std::mem::transmute(self._req) }
    }

    pub fn response(&mut self) -> &mut Response {
        unsafe { std::mem::transmute(self._resp) }
    }
}
