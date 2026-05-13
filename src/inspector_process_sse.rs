use crate::inspector::BodyInspector;
use bytes::{Bytes, BytesMut};

impl BodyInspector {
    pub(crate) fn process_sse(buffer: &mut BytesMut) -> Option<Bytes> {
        let mut validated_data = Vec::new();

        while let Some(pos) = memchr::memmem::find(buffer, b"\n\n") {
            let end_of_event = pos + 2;
            let event = buffer.split_to(end_of_event); // This is a BytesMut

            let mut modified_event = Vec::with_capacity(event.len() * 2); // Pre-allocate extra space

            for &byte in event.iter() {
                if byte == b'2' {
                    modified_event.extend_from_slice(b"TWO");
                } else {
                    modified_event.push(byte);
                }
            }

            validated_data.extend_from_slice(&modified_event);
        }

        if validated_data.is_empty() {
            None
        } else {
            Some(Bytes::from(validated_data))
        }
    }
}
