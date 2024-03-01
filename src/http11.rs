use tracing::trace;

use crate::{ctx::ConnContext, message::Message, protocols::Protocol, request::Request};

pub(crate) async fn serve<
    R: tokio::io::AsyncBufReadExt + Unpin,
    W: tokio::io::AsyncWriteExt + Unpin,
>(
    ctx: &mut ConnContext<R, W>,
) -> Protocol {
    let mut request = Request::default();
    let mut response = Message::default();

    loop {
        match (&mut (request.msg)).from11(ctx).await {
            crate::message::MessageReadCode::Ok => {
                on_request(ctx, &mut request, &mut response).await;
                break;
            }
            e => {
                #[cfg(debug_assertions)]
                {
                    trace!("read http message failed, {}, {:?}", ctx.addr, e);
                }
                break;
            }
        }
    }

    Protocol::None
}

async fn on_request<R: tokio::io::AsyncBufReadExt + Unpin, W: tokio::io::AsyncWriteExt + Unpin>(
    ctx: &mut ConnContext<R, W>,
    req: &mut Request,
    resp: &mut Message,
) {
    trace!(
        "request: $method: {} $url: {} $version: {}",
        req.method(),
        req.url(),
        req.version()
    );
}
