use crate::inspector::BodyInspector;
use bytes::{Bytes, BytesMut};

impl BodyInspector {
    pub fn process_standard_body(buffer: &mut BytesMut) -> Bytes {
        let data = buffer.split().to_vec();
        let text = String::from_utf8_lossy(&data);
        let modified = text.replace("2", "TWO");
        Bytes::from(modified)
    }
}
