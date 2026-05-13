use bytes::BytesMut;

pub struct BodyCtx {
    pub buffer: BytesMut,
    pub is_sse: bool,
}
