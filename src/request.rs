use crate::{
    error::{InvalidObserve, MessageError},
    packet::{CoapOption, MessageClass, ObserveOption, Packet, RequestType},
    PATH_MAX_SIZE,
};
use core::convert::TryFrom;
use heapless::String;

/// The CoAP request.
#[derive(Clone, Debug, PartialEq)]
pub struct CoapRequest<'a, Endpoint> {
    pub message: &'a Packet<'a>,
    pub source: Option<Endpoint>,
}

impl<'a, Endpoint> CoapRequest<'a, Endpoint> {
    pub fn from_packet<'b>(packet: &'b Packet<'b>, source: Endpoint) -> CoapRequest<'b, Endpoint> {
        CoapRequest {
            message: packet,
            source: Some(source),
        }
    }

    pub fn get_method(&self) -> &RequestType {
        match self.message.get_code() {
            MessageClass::Request(RequestType::Get) => &RequestType::Get,
            MessageClass::Request(RequestType::Post) => &RequestType::Post,
            MessageClass::Request(RequestType::Put) => &RequestType::Put,
            MessageClass::Request(RequestType::Delete) => &RequestType::Delete,
            MessageClass::Request(RequestType::Fetch) => &RequestType::Fetch,
            MessageClass::Request(RequestType::Patch) => &RequestType::Patch,
            MessageClass::Request(RequestType::IPatch) => &RequestType::IPatch,
            _ => &RequestType::UnKnown,
        }
    }

    pub fn get_path(&self) -> Result<String<PATH_MAX_SIZE>, MessageError> {
        let mut s = String::<PATH_MAX_SIZE>::new();
        for option_pair in self.message.get_options(CoapOption::UriPath) {
            match s.push_str(core::str::from_utf8(option_pair.value).unwrap()) {
                Err(_) => {
                    return Err(MessageError::PathLengthExceeded);
                }
                _ => {}
            }
        }
        return Ok(s);
    }

    /// Returns the flag in the Observe option or InvalidObserve if the flag
    /// was provided but not understood.
    pub fn get_observe_flag(&self) -> Option<Result<ObserveOption, InvalidObserve>> {
        self.message
            .get_observe_value()
            .map(|value| usize::try_from(value).unwrap())
            .map_or(Some(Err(InvalidObserve)), |value| {
                Some(ObserveOption::try_from(value))
            })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::packet::{MessageType, OptionPair};
    use heapless::Vec;

    struct Endpoint(());

    #[test]
    fn test_request_create() {
        let options = &[OptionPair {
            num: CoapOption::UriPath.into(),
            value: b"test-interface",
        }];
        let packet = Packet::new(
            MessageType::Confirmable,
            MessageClass::Request(RequestType::Get),
            /* version= */ 2,
            /* message_id= */ 42,
            /* token= */ &[0x17, 0x38],
            /* options= */ &mut Vec::from_slice(options).unwrap(),
            /* payload= */ "Hello".as_bytes(),
        );
        let request = CoapRequest::from_packet(&packet, Endpoint(()));

        assert!(request.source.is_some())
    }

    #[test]
    fn path_length_exceeded() {
        let buf = [
            0x40, 0x01, 0x00, 0x00, 0xbd, 0x77, 0x2f, 0x73, 0x6f, 0x6d, 0x65, 0x2d, 0x73, 0x75,
            0x70, 0x65, 0x72, 0x2d, 0x6c, 0x6f, 0x6e, 0x67, 0x2d, 0x70, 0x61, 0x74, 0x68, 0x31,
            0x2f, 0x73, 0x6f, 0x6d, 0x65, 0x2d, 0x73, 0x75, 0x70, 0x65, 0x72, 0x2d, 0x6c, 0x6f,
            0x6e, 0x67, 0x2d, 0x70, 0x61, 0x74, 0x68, 0x32, 0x2f, 0x73, 0x6f, 0x6d, 0x65, 0x2d,
            0x73, 0x75, 0x70, 0x65, 0x72, 0x2d, 0x6c, 0x6f, 0x6e, 0x67, 0x2d, 0x70, 0x61, 0x74,
            0x68, 0x33, 0x2f, 0x73, 0x6f, 0x6d, 0x65, 0x2d, 0x73, 0x75, 0x70, 0x65, 0x72, 0x2d,
            0x6c, 0x6f, 0x6e, 0x67, 0x2d, 0x70, 0x61, 0x74, 0x68, 0x34, 0x2f, 0x73, 0x6f, 0x6d,
            0x65, 0x2d, 0x73, 0x75, 0x70, 0x65, 0x72, 0x2d, 0x6c, 0x6f, 0x6e, 0x67, 0x2d, 0x70,
            0x61, 0x74, 0x68, 0x35, 0x2f, 0x73, 0x6f, 0x6d, 0x65, 0x2d, 0x73, 0x75, 0x70, 0x65,
            0x72, 0x2d, 0x6c, 0x6f, 0x6e, 0x67, 0x2d, 0x70, 0x61, 0x74, 0x68, 0x36,
        ];
        let p = Packet::from_bytes(&buf).unwrap();
        let request = CoapRequest::from_packet(&p, Endpoint(()));
        assert_eq!(
            MessageError::PathLengthExceeded,
            request.get_path().unwrap_err()
        );
    }
}
