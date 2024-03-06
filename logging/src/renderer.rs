use crate::event::Event;

pub trait Renderer {
    fn render(&self, evt: &Event, buf: &Vec<u8>);
}
