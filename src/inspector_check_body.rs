use crate::inspector::BodyInspector;
use bytes::BytesMut;
use pingora::ErrorType;

impl BodyInspector {
    pub(crate) fn check_body(body: &BytesMut) -> pingora::Result<()> {
        if memchr::memmem::find(body, b"rogue").is_some() {
            return Err(pingora::Error::new(ErrorType::Custom(
                "SecurityPolicyViolation",
            )));
        }
        Ok(())
    }
}
