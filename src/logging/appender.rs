use super::Item;

pub trait Renderer: Send + Sync {
    fn name(&self) -> &str;
    fn render(&self, item: &Item, buf: &mut Vec<u8>);
}

pub trait Filter: Send + Sync {
    fn filter(&self, item: &Item) -> bool;
}

struct FnFilter<F: Fn(&Item) -> bool + Send + Sync + 'static> {
    f: F,
}

impl<F: Fn(&Item) -> bool + Send + Sync + 'static> Filter for FnFilter<F> {
    fn filter(&self, item: &Item) -> bool {
        (self.f)(item)
    }
}

pub fn filter<T: Fn(&Item) -> bool + Send + Sync + 'static>(f: T) -> Box<dyn Filter> {
    Box::new(FnFilter { f })
}

#[async_trait::async_trait]
pub trait Appender: Send + Sync {
    async fn writeall(&mut self, buf: &[u8]) -> std::io::Result<()>;
    async fn flush(&mut self) -> std::io::Result<()>;

    fn service(&self) -> usize; // service name
    fn renderer(&self) -> &str; // renderer name
    fn filter(&self, item: &Item) -> bool;
}
