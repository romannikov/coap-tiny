use crate::packet::{MessageClass, Packet, ResponseType};

/// The CoAP response.
#[derive(Clone, Debug, PartialEq)]
pub struct CoapResponse<'a> {
    pub message: &'a Packet<'a>,
}

impl<'a> CoapResponse<'a> {
    /// Creates a new response.
    pub fn new<'b>(packet: &'b Packet) -> CoapResponse<'b> {
        CoapResponse { message: packet }
    }

    /// Returns the status.
    pub fn get_status(&self) -> &ResponseType {
        match self.message.get_code() {
            MessageClass::Response(ResponseType::Created) => &ResponseType::Created,
            MessageClass::Response(ResponseType::Deleted) => &ResponseType::Deleted,
            MessageClass::Response(ResponseType::Valid) => &ResponseType::Valid,
            MessageClass::Response(ResponseType::Changed) => &ResponseType::Changed,
            MessageClass::Response(ResponseType::Content) => &ResponseType::Content,

            MessageClass::Response(ResponseType::BadRequest) => &ResponseType::BadRequest,
            MessageClass::Response(ResponseType::Unauthorized) => &ResponseType::Unauthorized,
            MessageClass::Response(ResponseType::BadOption) => &ResponseType::BadOption,
            MessageClass::Response(ResponseType::Forbidden) => &ResponseType::Forbidden,
            MessageClass::Response(ResponseType::NotFound) => &ResponseType::NotFound,
            MessageClass::Response(ResponseType::MethodNotAllowed) => {
                &ResponseType::MethodNotAllowed
            }
            MessageClass::Response(ResponseType::NotAcceptable) => &ResponseType::NotAcceptable,
            MessageClass::Response(ResponseType::PreconditionFailed) => {
                &ResponseType::PreconditionFailed
            }
            MessageClass::Response(ResponseType::RequestEntityTooLarge) => {
                &ResponseType::RequestEntityTooLarge
            }
            MessageClass::Response(ResponseType::UnsupportedContentFormat) => {
                &ResponseType::UnsupportedContentFormat
            }

            MessageClass::Response(ResponseType::InternalServerError) => {
                &ResponseType::InternalServerError
            }
            MessageClass::Response(ResponseType::NotImplemented) => &ResponseType::NotImplemented,
            MessageClass::Response(ResponseType::BadGateway) => &ResponseType::BadGateway,
            MessageClass::Response(ResponseType::ServiceUnavailable) => {
                &ResponseType::ServiceUnavailable
            }
            MessageClass::Response(ResponseType::GatewayTimeout) => &ResponseType::GatewayTimeout,
            MessageClass::Response(ResponseType::ProxyingNotSupported) => {
                &ResponseType::ProxyingNotSupported
            }
            _ => &ResponseType::UnKnown,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::packet::MessageType;
    use heapless::Vec;

    #[test]
    fn test_new_response() {
        let packet = Packet::new(
            MessageType::Confirmable,
            MessageClass::Response(ResponseType::Content),
            /* version= */ 2,
            /* message_id= */ 42,
            /* token= */ &[0x17, 0x38],
            /* options= */ &mut Vec::new(),
            /* payload= */ "Hello".as_bytes(),
        );
        let opt_resp = CoapResponse::new(&packet);
        assert_eq!(opt_resp.get_status(), &ResponseType::Content);
    }
}
