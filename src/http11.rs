use tokio::io::{AsyncBufReadExt, AsyncReadExt};

use crate::{config::Config, ctx::ConnContext, message::Message, protocols::Protocol};

pub(crate) async fn serve<
    R: tokio::io::AsyncBufReadExt + Unpin,
    W: tokio::io::AsyncWriteExt + Unpin,
>(
    ctx: &mut ConnContext<R, W>,
) -> Protocol {
    let mut request = Message::default();
    let mut response = Message::default();

    loop {
        let code = (&mut request).from11(ctx).await;
    }

    Protocol::None
}
