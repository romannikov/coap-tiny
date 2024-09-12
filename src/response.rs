use crate::packet::{MessageClass, Packet, ResponseType};

/// The CoAP response.
#[derive(Clone, Debug, PartialEq)]
pub struct CoapResponse<'a> {
    pub message: &'a Packet<'a>,
}

impl<'a> CoapResponse<'a> {
    /// Creates a new response.
    pub fn from_packet<'b>(packet: &'b Packet) -> CoapResponse<'b> {
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

    #[test]
    fn test_new_response_valid() {
        for mtyp in [MessageType::Confirmable, MessageType::NonConfirmable] {
            let mut packet = Packet::new();
            packet.header.set_type(mtyp);
            let opt_resp = CoapResponse::new(&packet);
            assert!(opt_resp.is_some());

            let response = opt_resp.unwrap();
            assert_eq!(packet.payload, response.message.payload);
        }
    }

    // #[test]
    // fn test_new_response_invalid() {
    //     let mut packet = Packet::new();
    //     packet.header.set_type(MessageType::Acknowledgement);
    //     assert!(CoapResponse::new(&packet).is_none());
    // }
}
