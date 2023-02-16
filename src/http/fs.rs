use super::{ctx::Context, handler::FutureType, Handler};

pub type FsIndexRenderFuncType =
    Box<dyn (Fn(&mut Context, &Vec<tokio::fs::DirEntry>) -> String) + Send + Sync>;

pub struct FsHandler {
    root: String,
    prefix: String,
    disable_index: bool,
    index_render: Option<FsIndexRenderFuncType>,
    max_read_cap: u64,
}

impl FsHandler {
    pub fn new() {
		
	}
}

impl Handler for FsHandler {
    fn handler<'a: 'b, 'b>(&self, ctx: &'a mut Context) -> FutureType<'b> {
        Box::pin(async {})
    }
}
