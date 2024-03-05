use crate::ctx::ConnContext;

pub(crate) async fn serve<
    R: tokio::io::AsyncBufReadExt + Unpin,
    W: tokio::io::AsyncWriteExt + Unpin,
>(
    ctx: &mut ConnContext<R, W>,
) {
}
