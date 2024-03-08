use crate::item::Item;

pub trait Renderer: Send + Sync {
    fn name(&self) -> &str;
    fn render(&self, item: &Item, buf: &mut Vec<u8>);
}

pub trait Appender: Send + Sync {
    fn renderer(&self) -> &str; // renderer name
    fn filter(&self, item: &Item) -> bool;
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;
    fn flush(&mut self) -> std::io::Result<()>;
}
