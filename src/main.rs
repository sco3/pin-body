use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use log::info;
use pingora::prelude::*;
use pingora::server::configuration::Opt;
use pingora::server::Server;
use pingora::upstreams::peer::HttpPeer;

pub struct BodyInspector;

pub struct BodyCtx {
    buffer: BytesMut,
}

impl BodyInspector {
    async fn check_body(body: &BytesMut) -> Result<()> {
        if memchr::memmem::find(body, b"rogue").is_some() {
            return Err(pingora::Error::new(ErrorType::Custom(
                "SecurityPolicyViolation",
            )));
        }

        Ok(())
    }
}

#[async_trait]
impl ProxyHttp for BodyInspector {
    type CTX = BodyCtx;

    fn new_ctx(&self) -> Self::CTX {
        BodyCtx {
            buffer: BytesMut::new(),
        }
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let peer = Box::new(HttpPeer::new(("localhost", 8080), false, "".to_string()));
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
            BodyInspector::check_body(&ctx.buffer).await?;
            *body = Some(ctx.buffer.split().freeze());
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let opt = Opt::parse_args();
    let mut my_server = Server::new(Some(opt))?;
    my_server.bootstrap();

    let mut my_proxy = http_proxy_service(&my_server.configuration, BodyInspector);
    my_proxy.add_tcp("0.0.0.0:6152");

    info!("Body Inspector running on 0.0.0.0:6152");
    my_server.add_service(my_proxy);
    my_server.run_forever();
}
