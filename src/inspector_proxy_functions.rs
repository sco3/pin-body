use crate::body_ctx::BodyCtx;
use crate::inspector::BodyInspector;
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use pingora::http::ResponseHeader;
use pingora::prelude::ProxyHttp;
use pingora::prelude::Session;
use pingora::upstreams::peer::HttpPeer;
use pingora::Result;
use std::time::Duration;

#[async_trait]
impl ProxyHttp for BodyInspector {
    type CTX = BodyCtx;

    fn new_ctx(&self) -> Self::CTX {
        BodyCtx {
            buffer: BytesMut::new(),
            is_sse: false,
        }
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut BodyCtx,
    ) -> Result<Box<HttpPeer>> {
        let peer = Box::new(
            HttpPeer::new(
                ("localhost", 8080),
                false,
                String::new(), //
            ), //
        );
        Ok(peer)
    }

    async fn request_body_filter(
        &self,
        _session: &mut Session,
        body: &mut Option<Bytes>,
        end_of_stream: bool,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        if let Some(chunk) = body.take() {
            ctx.buffer.extend_from_slice(&chunk);
        }

        if end_of_stream {
            BodyInspector::check_body(&ctx.buffer)?;
            *body = Some(ctx.buffer.split().freeze());
        }

        Ok(())
    }

    async fn upstream_response_filter(
        &self,
        _session: &mut Session,
        upstream_res: &mut ResponseHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        if let Some(ct) = upstream_res.headers.get("Content-Type") {
            if ct == "text/event-stream" {
                ctx.is_sse = true;
            }
        }
        Ok(())
    }

    fn response_body_filter(
        &self,
        _session: &mut Session,
        body: &mut Option<Bytes>,
        end_of_stream: bool, // This is key!
        ctx: &mut Self::CTX,
    ) -> Result<Option<Duration>> {
        if let Some(chunk) = body.take() {
            ctx.buffer.extend_from_slice(&chunk);
        }

        if ctx.is_sse {
            *body = BodyInspector::process_sse(&mut ctx.buffer);
        } else if end_of_stream {
            *body = Some(BodyInspector::process_standard_body(&mut ctx.buffer));
        } else {
            *body = None;
        }
        Ok(None)
    }
}
