use core::convert::TryFrom;

use heapless::String;

use crate::{
    error::InvalidObserve,
    packet::{CoapOption, MessageClass, ObserveOption, Packet, RequestType},
    PATH_MAX_SIZE,
};

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

    pub fn get_path(&self) -> String<PATH_MAX_SIZE> {
        let mut s = String::<PATH_MAX_SIZE>::new();
        for option_pair in self.message.get_options(CoapOption::UriPath) {
            let _ = s.push_str(core::str::from_utf8(option_pair.value).unwrap());
        }
        return s;
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
    use core::str::FromStr;

    use heapless::Vec;

    use super::*;
    use crate::packet::{MessageType, OptionPair};

    struct Endpoint(String<20>);

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
        let endpoint = Endpoint(String::from_str("127.0.0.1:1234").unwrap());
        let request = CoapRequest::from_packet(&packet, endpoint);

        assert!(request.source.is_some())
    }
}
