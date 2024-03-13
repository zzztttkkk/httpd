use crate::item::Item;

pub trait Renderer: Send + Sync {
    fn name(&self) -> &str;
    fn render(&self, item: &Item, buf: &mut Vec<u8>);
}

#[async_trait::async_trait]
pub trait Appender: Send + Sync {
    async fn writeall(&mut self, buf: &[u8]) -> std::io::Result<()>;
    async fn flush(&mut self) -> std::io::Result<()>;
    fn renderer(&self) -> &str; // renderer name
    fn filter(&self, item: &Item) -> bool;
}

pub type FilterFn = Box<dyn Fn(&Item) -> bool + Send + Sync + Unpin + 'static>;
