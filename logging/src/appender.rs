use crate::item::Item;

pub trait Renderer: Send + Sync {
    fn name(&self) -> &str;
    fn render(&self, item: &Item, buf: &mut Vec<u8>);
}

pub trait Appender: tokio::io::AsyncWrite + Unpin + Send + Sync {
    fn renderer(&self) -> &str; // renderer name
    fn filter(&self, item: &Item) -> bool;
}

pub type FilterFn = Box<dyn Fn(&Item) -> bool + Send + Sync + Unpin + 'static>;
