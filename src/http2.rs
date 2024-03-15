use crate::{ctx::ConnContext, message::Message};

pub(crate) async fn serve<
    R: tokio::io::AsyncBufReadExt + Unpin,
    W: tokio::io::AsyncWriteExt + Unpin,
>(
    ctx: ConnContext<R, W>,
    req: Message,
) {
}
