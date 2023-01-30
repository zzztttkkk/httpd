use std::{any::Any, collections::HashMap};

#[derive(Debug)]
pub struct Context(HashMap<String, Box<dyn Any>>);

unsafe impl Send for Context {}

unsafe impl Sync for Context {}

impl Context {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get<T: Any>(&self, k: &str) -> Option<&T> {
        match self.0.get(k) {
            Some(v) => v.downcast_ref(),
            None => None,
        }
    }

    pub fn get_mut<T: Any>(&mut self, k: &str) -> Option<&mut T> {
        match self.0.get_mut(k) {
            Some(v) => v.downcast_mut(),
            None => None,
        }
    }

    pub fn set(&mut self, k: &str, v: Box<dyn Any>) {
        self.0.insert(k.to_string(), v);
    }
}

mod tests {
    use std::any::Any;

    use super::Context;

    #[test]
    fn test_ctx() {
        let mut ctx = Context::new();
        ctx.set("begin", Box::new(std::time::SystemTime::now()));

        let begin = ctx.get::<std::time::SystemTime>("begin").unwrap();

        println!(
            "{} {:?}",
            std::time::SystemTime::now()
                .duration_since(*begin)
                .unwrap()
                .as_micros(),
            Any::type_id(&12)
        );
    }
}
