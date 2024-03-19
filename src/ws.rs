use crate::{ctx::ConnContext, message::Message};

pub(crate) async fn serve<
    R: tokio::io::AsyncBufReadExt + Unpin,
    W: tokio::io::AsyncWriteExt + Unpin,
>(
    _ctx: ConnContext<R, W>,
    _req: Message,
) {
}
