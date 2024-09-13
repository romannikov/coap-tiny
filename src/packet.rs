use crate::error::{
    IncompatibleOptionValueFormat, InvalidContentFormat, InvalidObserve, MessageError,
};
use crate::{MAX_OPTIONS, PACKET_MAX_SIZE};
use core::{convert::TryFrom, fmt::Write};
use heapless::{String, Vec};

macro_rules! u8_to_unsigned_be {
    ($src:ident, $start:expr, $end:expr, $t:ty) => ({
        (0..=$end - $start).rev().fold(
            0, |acc, i| acc | $src[$start+i] as $t << i * 8
        )
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageClass {
    Empty,
    Request(RequestType),
    Response(ResponseType),
    Reserved(u8),
}

impl From<u8> for MessageClass {
    fn from(number: u8) -> MessageClass {
        match number {
            0x00 => MessageClass::Empty,

            0x01 => MessageClass::Request(RequestType::Get),
            0x02 => MessageClass::Request(RequestType::Post),
            0x03 => MessageClass::Request(RequestType::Put),
            0x04 => MessageClass::Request(RequestType::Delete),
            0x05 => MessageClass::Request(RequestType::Fetch),
            0x06 => MessageClass::Request(RequestType::Patch),
            0x07 => MessageClass::Request(RequestType::IPatch),

            0x41 => MessageClass::Response(ResponseType::Created),
            0x42 => MessageClass::Response(ResponseType::Deleted),
            0x43 => MessageClass::Response(ResponseType::Valid),
            0x44 => MessageClass::Response(ResponseType::Changed),
            0x45 => MessageClass::Response(ResponseType::Content),
            0x5F => MessageClass::Response(ResponseType::Continue),

            0x80 => MessageClass::Response(ResponseType::BadRequest),
            0x81 => MessageClass::Response(ResponseType::Unauthorized),
            0x82 => MessageClass::Response(ResponseType::BadOption),
            0x83 => MessageClass::Response(ResponseType::Forbidden),
            0x84 => MessageClass::Response(ResponseType::NotFound),
            0x85 => MessageClass::Response(ResponseType::MethodNotAllowed),
            0x86 => MessageClass::Response(ResponseType::NotAcceptable),
            0x89 => MessageClass::Response(ResponseType::Conflict),
            0x8C => MessageClass::Response(ResponseType::PreconditionFailed),
            0x8D => MessageClass::Response(ResponseType::RequestEntityTooLarge),
            0x8F => MessageClass::Response(ResponseType::UnsupportedContentFormat),
            0x88 => MessageClass::Response(ResponseType::RequestEntityIncomplete),
            0x96 => MessageClass::Response(ResponseType::UnprocessableEntity),
            0x9d => MessageClass::Response(ResponseType::TooManyRequests),

            0xA0 => MessageClass::Response(ResponseType::InternalServerError),
            0xA1 => MessageClass::Response(ResponseType::NotImplemented),
            0xA2 => MessageClass::Response(ResponseType::BadGateway),
            0xA3 => MessageClass::Response(ResponseType::ServiceUnavailable),
            0xA4 => MessageClass::Response(ResponseType::GatewayTimeout),
            0xA5 => MessageClass::Response(ResponseType::ProxyingNotSupported),
            0xA8 => MessageClass::Response(ResponseType::HopLimitReached),

            n => MessageClass::Reserved(n),
        }
    }
}

impl From<MessageClass> for u8 {
    fn from(class: MessageClass) -> u8 {
        match class {
            MessageClass::Empty => 0x00,

            MessageClass::Request(RequestType::Get) => 0x01,
            MessageClass::Request(RequestType::Post) => 0x02,
            MessageClass::Request(RequestType::Put) => 0x03,
            MessageClass::Request(RequestType::Delete) => 0x04,
            MessageClass::Request(RequestType::Fetch) => 0x05,
            MessageClass::Request(RequestType::Patch) => 0x06,
            MessageClass::Request(RequestType::IPatch) => 0x07,
            MessageClass::Request(RequestType::UnKnown) => 0xFF,

            MessageClass::Response(ResponseType::Created) => 0x41,
            MessageClass::Response(ResponseType::Deleted) => 0x42,
            MessageClass::Response(ResponseType::Valid) => 0x43,
            MessageClass::Response(ResponseType::Changed) => 0x44,
            MessageClass::Response(ResponseType::Content) => 0x45,
            MessageClass::Response(ResponseType::Continue) => 0x5F,

            MessageClass::Response(ResponseType::BadRequest) => 0x80,
            MessageClass::Response(ResponseType::Unauthorized) => 0x81,
            MessageClass::Response(ResponseType::BadOption) => 0x82,
            MessageClass::Response(ResponseType::Forbidden) => 0x83,
            MessageClass::Response(ResponseType::NotFound) => 0x84,
            MessageClass::Response(ResponseType::MethodNotAllowed) => 0x85,
            MessageClass::Response(ResponseType::NotAcceptable) => 0x86,
            MessageClass::Response(ResponseType::Conflict) => 0x89,
            MessageClass::Response(ResponseType::PreconditionFailed) => 0x8C,
            MessageClass::Response(ResponseType::RequestEntityTooLarge) => 0x8D,
            MessageClass::Response(ResponseType::UnsupportedContentFormat) => 0x8F,
            MessageClass::Response(ResponseType::RequestEntityIncomplete) => 0x88,
            MessageClass::Response(ResponseType::UnprocessableEntity) => 0x96,
            MessageClass::Response(ResponseType::TooManyRequests) => 0x9d,

            MessageClass::Response(ResponseType::InternalServerError) => 0xA0,
            MessageClass::Response(ResponseType::NotImplemented) => 0xA1,
            MessageClass::Response(ResponseType::BadGateway) => 0xA2,
            MessageClass::Response(ResponseType::ServiceUnavailable) => 0xA3,
            MessageClass::Response(ResponseType::GatewayTimeout) => 0xA4,
            MessageClass::Response(ResponseType::ProxyingNotSupported) => 0xA5,
            MessageClass::Response(ResponseType::HopLimitReached) => 0xA8,
            MessageClass::Response(ResponseType::UnKnown) => 0xFF,

            MessageClass::Reserved(c) => c,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RequestType {
    Get,
    Post,
    Put,
    Delete,
    Fetch,
    Patch,
    IPatch,
    UnKnown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResponseType {
    // 200 Codes
    Created,
    Deleted,
    Valid,
    Changed,
    Content,
    Continue,

    // 400 Codes
    BadRequest,
    Unauthorized,
    BadOption,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAcceptable,
    Conflict,
    PreconditionFailed,
    RequestEntityTooLarge,
    UnsupportedContentFormat,
    RequestEntityIncomplete,
    UnprocessableEntity,
    TooManyRequests,

    // 500 Codes
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    ProxyingNotSupported,
    HopLimitReached,

    UnKnown,
}

/// CoAP request/response message type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
    Confirmable,
    NonConfirmable,
    Acknowledgement,
    Reset,
}

/// The CoAP options.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoapOption {
    IfMatch,
    UriHost,
    ETag,
    IfNoneMatch,
    Observe,
    UriPort,
    LocationPath,
    Oscore,
    UriPath,
    ContentFormat,
    MaxAge,
    UriQuery,
    Accept,
    LocationQuery,
    Block2,
    Block1,
    ProxyUri,
    ProxyScheme,
    Size1,
    Size2,
    NoResponse,
    Unknown(u16),
}

impl From<u16> for CoapOption {
    fn from(number: u16) -> CoapOption {
        match number {
            1 => CoapOption::IfMatch,
            3 => CoapOption::UriHost,
            4 => CoapOption::ETag,
            5 => CoapOption::IfNoneMatch,
            6 => CoapOption::Observe,
            7 => CoapOption::UriPort,
            8 => CoapOption::LocationPath,
            9 => CoapOption::Oscore,
            11 => CoapOption::UriPath,
            12 => CoapOption::ContentFormat,
            14 => CoapOption::MaxAge,
            15 => CoapOption::UriQuery,
            17 => CoapOption::Accept,
            20 => CoapOption::LocationQuery,
            23 => CoapOption::Block2,
            27 => CoapOption::Block1,
            35 => CoapOption::ProxyUri,
            39 => CoapOption::ProxyScheme,
            60 => CoapOption::Size1,
            28 => CoapOption::Size2,
            258 => CoapOption::NoResponse,
            _ => CoapOption::Unknown(number),
        }
    }
}

impl From<CoapOption> for u16 {
    fn from(option: CoapOption) -> u16 {
        match option {
            CoapOption::IfMatch => 1,
            CoapOption::UriHost => 3,
            CoapOption::ETag => 4,
            CoapOption::IfNoneMatch => 5,
            CoapOption::Observe => 6,
            CoapOption::UriPort => 7,
            CoapOption::LocationPath => 8,
            CoapOption::Oscore => 9,
            CoapOption::UriPath => 11,
            CoapOption::ContentFormat => 12,
            CoapOption::MaxAge => 14,
            CoapOption::UriQuery => 15,
            CoapOption::Accept => 17,
            CoapOption::LocationQuery => 20,
            CoapOption::Block2 => 23,
            CoapOption::Block1 => 27,
            CoapOption::ProxyUri => 35,
            CoapOption::ProxyScheme => 39,
            CoapOption::Size1 => 60,
            CoapOption::Size2 => 28,
            CoapOption::NoResponse => 258,
            CoapOption::Unknown(number) => number,
        }
    }
}

/// The content formats.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum ContentFormat {
    TextPlain,
    // Media-Type: `application/cose; cose-type="cose-encrypt0"`, ID: 16
    ApplicationCoseEncrypt0,
    // Media-Type: `application/cose; cose-type="cose-mac0"`, ID: 17
    ApplicationCoseMac0,
    // Media-Type: `application/cose; cose-type="cose-sign1"`, ID: 18
    ApplicationCoseSign1,
    ApplicationAceCbor,
    ImageGif,
    ImageJpeg,
    ImagePng,
    ApplicationLinkFormat,
    ApplicationXML,
    ApplicationOctetStream,
    ApplicationEXI,
    ApplicationJSON,
    ApplicationJsonPatchJson,
    ApplicationMergePatchJson,
    ApplicationCBOR,
    ApplicationCWt,
    ApplicationMultipartCore,
    ApplicationCborSeq,
    // Media-Type: `application/cose; cose-type="cose-encrypt"`, ID: 96
    ApplicationCoseEncrypt,
    // Media-Type: `application/cose; cose-type="cose-mac"`, ID: 97
    ApplicationCoseMac,
    // Media-Type: `application/cose; cose-type="cose-sign"`, ID: 98
    ApplicationCoseSign,
    ApplicationCoseKey,
    ApplicationCoseKeySet,
    ApplicationSenmlJSON,
    ApplicationSensmlJSON,
    ApplicationSenmlCBOR,
    ApplicationSensmlCBOR,
    ApplicationSenmlExi,
    ApplicationSensmlExi,
    // Media-Type: `application/yang-data+cbor; id=sid`, ID: 140
    ApplicationYangDataCborSid,
    ApplicationCoapGroupJson,
    ApplicationDotsCbor,
    ApplicationMissingBlocksCborSeq,
    // Media-Type: `application/pkcs7-mime; smime-type=server-generated-key`, ID: 280
    ApplicationPkcs7MimeServerGeneratedKey,
    // Media-Type: `application/pkcs7-mime; smime-type=certs-only`, ID: 281
    ApplicationPkcs7MimeCertsOnly,
    ApplicationPkcs8,
    ApplicationCsrattrs,
    ApplicationPkcs10,
    ApplicationPkixCert,
    ApplicationAifCbor,
    ApplicationAifJson,
    ApplicationSenmlXML,
    ApplicationSensmlXML,
    ApplicationSenmlEtchJson,
    ApplicationSenmlEtchCbor,
    ApplicationYangDataCbor,
    // Media-Type: `application/yang-data+cbor; id=name`, ID: 341
    ApplicationYangDataCborName,
    ApplicationTdJson,
    ApplicationVoucherCoseCbor,
    ApplicationVndOcfCbor,
    ApplicationOscore,
    ApplicationJavascript,
    ApplicationJsonDeflate,
    ApplicationCborDeflate,
    ApplicationVndOmaLwm2mTlv,
    ApplicationVndOmaLwm2mJson,
    ApplicationVndOmaLwm2mCbor,
    TextCss,
    ImageSvgXml,
}

impl TryFrom<usize> for ContentFormat {
    type Error = InvalidContentFormat;

    fn try_from(number: usize) -> Result<ContentFormat, InvalidContentFormat> {
        match number {
            0 => Ok(ContentFormat::TextPlain),
            16 => Ok(ContentFormat::ApplicationCoseEncrypt0),
            17 => Ok(ContentFormat::ApplicationCoseMac0),
            18 => Ok(ContentFormat::ApplicationCoseSign1),
            19 => Ok(ContentFormat::ApplicationAceCbor),
            21 => Ok(ContentFormat::ImageGif),
            22 => Ok(ContentFormat::ImageJpeg),
            23 => Ok(ContentFormat::ImagePng),
            40 => Ok(ContentFormat::ApplicationLinkFormat),
            41 => Ok(ContentFormat::ApplicationXML),
            42 => Ok(ContentFormat::ApplicationOctetStream),
            47 => Ok(ContentFormat::ApplicationEXI),
            50 => Ok(ContentFormat::ApplicationJSON),
            51 => Ok(ContentFormat::ApplicationJsonPatchJson),
            52 => Ok(ContentFormat::ApplicationMergePatchJson),
            60 => Ok(ContentFormat::ApplicationCBOR),
            61 => Ok(ContentFormat::ApplicationCWt),
            62 => Ok(ContentFormat::ApplicationMultipartCore),
            63 => Ok(ContentFormat::ApplicationCborSeq),
            96 => Ok(ContentFormat::ApplicationCoseEncrypt),
            97 => Ok(ContentFormat::ApplicationCoseMac),
            98 => Ok(ContentFormat::ApplicationCoseSign),
            101 => Ok(ContentFormat::ApplicationCoseKey),
            102 => Ok(ContentFormat::ApplicationCoseKeySet),
            110 => Ok(ContentFormat::ApplicationSenmlJSON),
            111 => Ok(ContentFormat::ApplicationSensmlJSON),
            112 => Ok(ContentFormat::ApplicationSenmlCBOR),
            113 => Ok(ContentFormat::ApplicationSensmlCBOR),
            114 => Ok(ContentFormat::ApplicationSenmlExi),
            115 => Ok(ContentFormat::ApplicationSensmlExi),
            140 => Ok(ContentFormat::ApplicationYangDataCborSid),
            256 => Ok(ContentFormat::ApplicationCoapGroupJson),
            271 => Ok(ContentFormat::ApplicationDotsCbor),
            272 => Ok(ContentFormat::ApplicationMissingBlocksCborSeq),
            280 => Ok(ContentFormat::ApplicationPkcs7MimeServerGeneratedKey),
            281 => Ok(ContentFormat::ApplicationPkcs7MimeCertsOnly),
            284 => Ok(ContentFormat::ApplicationPkcs8),
            285 => Ok(ContentFormat::ApplicationCsrattrs),
            286 => Ok(ContentFormat::ApplicationPkcs10),
            287 => Ok(ContentFormat::ApplicationPkixCert),
            290 => Ok(ContentFormat::ApplicationAifCbor),
            291 => Ok(ContentFormat::ApplicationAifJson),
            310 => Ok(ContentFormat::ApplicationSenmlXML),
            311 => Ok(ContentFormat::ApplicationSensmlXML),
            320 => Ok(ContentFormat::ApplicationSenmlEtchJson),
            322 => Ok(ContentFormat::ApplicationSenmlEtchCbor),
            340 => Ok(ContentFormat::ApplicationYangDataCbor),
            341 => Ok(ContentFormat::ApplicationYangDataCborName),
            432 => Ok(ContentFormat::ApplicationTdJson),
            836 => Ok(ContentFormat::ApplicationVoucherCoseCbor),
            10000 => Ok(ContentFormat::ApplicationVndOcfCbor),
            10001 => Ok(ContentFormat::ApplicationOscore),
            10002 => Ok(ContentFormat::ApplicationJavascript),
            11050 => Ok(ContentFormat::ApplicationJsonDeflate),
            11060 => Ok(ContentFormat::ApplicationCborDeflate),
            11542 => Ok(ContentFormat::ApplicationVndOmaLwm2mTlv),
            11543 => Ok(ContentFormat::ApplicationVndOmaLwm2mJson),
            11544 => Ok(ContentFormat::ApplicationVndOmaLwm2mCbor),
            20000 => Ok(ContentFormat::TextCss),
            30000 => Ok(ContentFormat::ImageSvgXml),
            _ => Err(InvalidContentFormat),
        }
    }
}

impl From<ContentFormat> for usize {
    fn from(format: ContentFormat) -> usize {
        match format {
            ContentFormat::TextPlain => 0,
            ContentFormat::ApplicationCoseEncrypt0 => 16,
            ContentFormat::ApplicationCoseMac0 => 17,
            ContentFormat::ApplicationCoseSign1 => 18,
            ContentFormat::ApplicationAceCbor => 19,
            ContentFormat::ImageGif => 21,
            ContentFormat::ImageJpeg => 22,
            ContentFormat::ImagePng => 23,
            ContentFormat::ApplicationLinkFormat => 40,
            ContentFormat::ApplicationXML => 41,
            ContentFormat::ApplicationOctetStream => 42,
            ContentFormat::ApplicationEXI => 47,
            ContentFormat::ApplicationJSON => 50,
            ContentFormat::ApplicationJsonPatchJson => 51,
            ContentFormat::ApplicationMergePatchJson => 52,
            ContentFormat::ApplicationCBOR => 60,
            ContentFormat::ApplicationCWt => 61,
            ContentFormat::ApplicationMultipartCore => 62,
            ContentFormat::ApplicationCborSeq => 63,
            ContentFormat::ApplicationCoseEncrypt => 96,
            ContentFormat::ApplicationCoseMac => 97,
            ContentFormat::ApplicationCoseSign => 98,
            ContentFormat::ApplicationCoseKey => 101,
            ContentFormat::ApplicationCoseKeySet => 102,
            ContentFormat::ApplicationSenmlJSON => 110,
            ContentFormat::ApplicationSensmlJSON => 111,
            ContentFormat::ApplicationSenmlCBOR => 112,
            ContentFormat::ApplicationSensmlCBOR => 113,
            ContentFormat::ApplicationSenmlExi => 114,
            ContentFormat::ApplicationSensmlExi => 115,
            ContentFormat::ApplicationYangDataCborSid => 140,
            ContentFormat::ApplicationCoapGroupJson => 256,
            ContentFormat::ApplicationDotsCbor => 271,
            ContentFormat::ApplicationMissingBlocksCborSeq => 272,
            ContentFormat::ApplicationPkcs7MimeServerGeneratedKey => 280,
            ContentFormat::ApplicationPkcs7MimeCertsOnly => 281,
            ContentFormat::ApplicationPkcs8 => 284,
            ContentFormat::ApplicationCsrattrs => 285,
            ContentFormat::ApplicationPkcs10 => 286,
            ContentFormat::ApplicationPkixCert => 287,
            ContentFormat::ApplicationAifCbor => 290,
            ContentFormat::ApplicationAifJson => 291,
            ContentFormat::ApplicationSenmlXML => 310,
            ContentFormat::ApplicationSensmlXML => 311,
            ContentFormat::ApplicationSenmlEtchJson => 320,
            ContentFormat::ApplicationSenmlEtchCbor => 322,
            ContentFormat::ApplicationYangDataCbor => 340,
            ContentFormat::ApplicationYangDataCborName => 341,
            ContentFormat::ApplicationTdJson => 432,
            ContentFormat::ApplicationVoucherCoseCbor => 836,
            ContentFormat::ApplicationVndOcfCbor => 10000,
            ContentFormat::ApplicationOscore => 10001,
            ContentFormat::ApplicationJavascript => 10002,
            ContentFormat::ApplicationJsonDeflate => 11050,
            ContentFormat::ApplicationCborDeflate => 11060,
            ContentFormat::ApplicationVndOmaLwm2mTlv => 11542,
            ContentFormat::ApplicationVndOmaLwm2mJson => 11543,
            ContentFormat::ApplicationVndOmaLwm2mCbor => 11544,
            ContentFormat::TextCss => 20000,
            ContentFormat::ImageSvgXml => 30000,
        }
    }
}

// The values of the observe option.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObserveOption {
    Register,
    Deregister,
}

impl TryFrom<usize> for ObserveOption {
    type Error = InvalidObserve;

    fn try_from(number: usize) -> Result<ObserveOption, InvalidObserve> {
        match number {
            0 => Ok(ObserveOption::Register),
            1 => Ok(ObserveOption::Deregister),
            _ => Err(InvalidObserve),
        }
    }
}

impl From<ObserveOption> for usize {
    fn from(observe: ObserveOption) -> usize {
        match observe {
            ObserveOption::Register => 0,
            ObserveOption::Deregister => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct OptionPair<'a> {
    pub num: u16,
    pub value: &'a [u8],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Packet<'a> {
    /// Version, message type and token length byte.
    ver_type_tkl: u8,
    code: MessageClass,
    message_id: u16,
    pub token: &'a [u8],
    /// Sorted by OptionPair.num vector of options.
    pub options: Vec<OptionPair<'a>, MAX_OPTIONS>,
    pub payload: &'a [u8],
}

impl<'a> Packet<'a> {
    pub fn new(
        t: MessageType,
        code: MessageClass,
        version: u8,
        message_id: u16,
        token: &'a [u8],
        options: &mut Vec<OptionPair<'a>, MAX_OPTIONS>,
        payload: &'a [u8],
    ) -> Self {
        let tn = match t {
            MessageType::Confirmable => 0,
            MessageType::NonConfirmable => 1,
            MessageType::Acknowledgement => 2,
            MessageType::Reset => 3,
        };
        // Set version.
        let mut ver_type_tkl = version << 6;
        // Set type.
        ver_type_tkl = tn << 4 | (0xCF & ver_type_tkl);
        // Set token length.
        assert_eq!(0xF0 & token.len(), 0);
        ver_type_tkl = (token.len() as u8) | (0xF0 & ver_type_tkl);

        Self::sort_option_pairs(options);

        Packet {
            ver_type_tkl,
            code,
            message_id,
            token,
            options: Vec::<OptionPair<'a>, MAX_OPTIONS>::from_iter(options.iter().cloned()),
            payload,
        }
    }

    fn sort_option_pairs<'b, const MAX_OPTIONS: usize>(vec: &mut Vec<OptionPair<'b>, MAX_OPTIONS>) {
        let len = vec.len();
        for i in 1..len {
            let mut j = i;
            while j > 0 && vec[j - 1].num > vec[j].num {
                vec.swap(j - 1, j);
                j -= 1;
            }
        }
    }

    #[inline]
    pub fn get_version(&self) -> u8 {
        self.ver_type_tkl >> 6
    }

    #[inline]
    pub fn get_type(&self) -> MessageType {
        let tn = (0x30 & self.ver_type_tkl) >> 4;
        match tn {
            0 => MessageType::Confirmable,
            1 => MessageType::NonConfirmable,
            2 => MessageType::Acknowledgement,
            3 => MessageType::Reset,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn get_token_length(&self) -> u8 {
        Self::get_token_length_internal(self.ver_type_tkl)
    }

    fn get_token_length_internal(ver_type_tkl: u8) -> u8 {
        0x0F & ver_type_tkl
    }

    #[inline]
    pub fn get_message_id(&self) -> u16 {
        self.message_id
    }

    pub fn options(&self) -> core::slice::Iter<'_, OptionPair<'_>> {
        self.options.iter()
    }

    pub fn get_code(&self) -> MessageClass {
        self.code
    }

    pub fn get_token(&self) -> &[u8] {
        &self.token
    }

    pub fn get_payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn get_options(&self, tp: CoapOption) -> impl Iterator<Item = &OptionPair<'a>> {
        self.options.iter().filter(move |&p| p.num == tp.into())
    }

    pub fn get_first_option(&self, tp: CoapOption) -> Option<&OptionPair<'a>> {
        self.options.iter().find(|&p| p.num == tp.into())
    }

    pub fn get_content_format_value(&self) -> Option<u16> {
        self.get_first_option(CoapOption::ContentFormat)
            .map(|option| self.to_uint::<u16>(option.value))
            .and_then(|value| value.ok())
    }

    pub fn get_observe_value(&self) -> Option<u32> {
        self.get_first_option(CoapOption::Observe)
            .map(|option| self.to_uint::<u32>(option.value))
            .and_then(|value| value.ok())
    }

    pub fn from_bytes<'b>(buf: &'b [u8]) -> Result<Packet<'b>, MessageError> {
        let header_result = Self::try_header(buf);
        if header_result.is_err() {
            return Err(header_result.unwrap_err());
        }
        let raw_header = header_result.unwrap();
        let token_length = Self::get_token_length_internal(raw_header.0);
        let options_start: usize = 4 + token_length as usize;

        if token_length > 8 {
            return Err(MessageError::InvalidTokenLength);
        }

        if options_start > buf.len() {
            return Err(MessageError::InvalidTokenLength);
        }
        let token = &buf[4..options_start];

        let mut idx = options_start;
        let mut options_number = 0;
        let mut options = Vec::<OptionPair, MAX_OPTIONS>::new();
        while idx < buf.len() {
            let byte = buf[idx];

            if byte == 255 || idx > buf.len() {
                break;
            }

            let mut delta = (byte >> 4) as u16;
            let mut length = (byte & 0xF) as usize;

            idx += 1;

            // Check for special delta characters
            match delta {
                13 => {
                    if idx >= buf.len() {
                        return Err(MessageError::InvalidOptionLength);
                    }
                    delta = (buf[idx] + 13).into();
                    idx += 1;
                }
                14 => {
                    if idx + 1 >= buf.len() {
                        return Err(MessageError::InvalidOptionLength);
                    }

                    delta = u16::from_be(u8_to_unsigned_be!(buf, idx, idx + 1, u16)) + 269;
                    idx += 2;
                }
                15 => {
                    return Err(MessageError::InvalidOptionDelta);
                }
                _ => {}
            };

            // Check for special length characters
            match length {
                13 => {
                    if idx >= buf.len() {
                        return Err(MessageError::InvalidOptionLength);
                    }

                    length = buf[idx] as usize + 13;
                    idx += 1;
                }
                14 => {
                    if idx + 1 >= buf.len() {
                        return Err(MessageError::InvalidOptionLength);
                    }

                    length =
                        (u16::from_be(u8_to_unsigned_be!(buf, idx, idx + 1, u16)) + 269) as usize;
                    idx += 2;
                }
                15 => {
                    return Err(MessageError::InvalidOptionLength);
                }
                _ => {}
            };

            options_number += delta;

            let end = idx + length;
            if end > buf.len() {
                return Err(MessageError::InvalidOptionLength);
            }
            match options.push(OptionPair {
                num: options_number,
                value: &buf[idx..end],
            }) {
                Err(_) => return Err(MessageError::OptionsLimitExceeded),
                _ => {}
            }

            idx += length;
        }

        let payload = if idx < buf.len() {
            &buf[(idx + 1)..buf.len()]
        } else {
            &[0; 0]
        };

        Ok(Packet {
            ver_type_tkl: raw_header.0,
            code: raw_header.1.into(),
            message_id: raw_header.2,
            token,
            options: options,
            payload: payload,
        })
    }

    fn try_header(buf: &[u8]) -> Result<(u8, u8, u16), MessageError> {
        if buf.len() < 4 {
            return Err(MessageError::InvalidPacketLength);
        }

        let mut id_bytes = [0; 2];
        id_bytes.copy_from_slice(&buf[2..4]);

        Ok((buf[0], buf[1], u16::from_be_bytes(id_bytes)))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8, PACKET_MAX_SIZE>, MessageError> {
        let mut options_delta_length = 0;
        let mut options_bytes: Vec<u8, PACKET_MAX_SIZE> = Vec::new();
        let mut i = 0;
        while i < self.options.len() {
            let start_option_pair = self.options.get(i);
            let mut j = i;
            while j < self.options.len()
                && start_option_pair.unwrap().num == self.options.get(j).unwrap().num
            {
                let value = self.options.get(j).unwrap().value;
                let mut header = Vec::<u8, 5>::new();
                let delta = start_option_pair.unwrap().num - options_delta_length;

                let mut byte: u8 = 0;
                if delta <= 12 {
                    byte |= (delta << 4) as u8;
                } else if delta < 269 {
                    byte |= 13 << 4;
                } else {
                    byte |= 14 << 4;
                }
                if value.len() <= 12 {
                    byte |= value.len() as u8;
                } else if value.len() < 269 {
                    byte |= 13;
                } else {
                    byte |= 14;
                }
                let _ = header.push(byte);

                if delta > 12 && delta < 269 {
                    let _ = header.push((delta - 13) as u8);
                } else if delta >= 269 {
                    let fix = delta - 269;
                    let _ = header.push((fix >> 8) as u8);
                    let _ = header.push((fix & 0xFF) as u8);
                }

                if value.len() > 12 && value.len() < 269 {
                    let _ = header.push((value.len() - 13) as u8);
                } else if value.len() >= 269 {
                    let fix = (value.len() - 269) as u16;
                    let _ = header.push((fix >> 8) as u8);
                    let _ = header.push((fix & 0xFF) as u8);
                }

                options_delta_length += delta;
                unsafe {
                    use core::ptr;
                    let buf_len = options_bytes.len();
                    ptr::copy(
                        header.as_ptr(),
                        options_bytes.as_mut_ptr().add(buf_len),
                        header.len(),
                    );
                    ptr::copy(
                        value.as_ptr(),
                        options_bytes.as_mut_ptr().add(buf_len + header.len()),
                        value.len(),
                    );
                    options_bytes.set_len(buf_len + header.len() + value.len());
                }
                j += 1;
            }
            i = j;
        }

        let mut buf_length = 4 + self.payload.len() + self.token.len();
        if self.get_code() != MessageClass::Empty && !self.payload.is_empty() {
            buf_length += 1;
        }
        buf_length += options_bytes.len();

        if PACKET_MAX_SIZE < buf_length {
            return Err(MessageError::InvalidPacketLength);
        }

        let mut buf = Vec::<u8, PACKET_MAX_SIZE>::new();
        let _ = buf.push(self.ver_type_tkl);
        let _ = buf.push(self.code.into());
        let id_bytes = self.message_id.to_be_bytes();
        buf.extend(id_bytes);

        unsafe {
            use core::ptr;
            let buf_len = buf.len();
            ptr::copy(
                self.token.as_ptr(),
                buf.as_mut_ptr().add(buf_len),
                self.token.len(),
            );
            ptr::copy(
                options_bytes.as_ptr(),
                buf.as_mut_ptr().add(buf_len + self.token.len()),
                options_bytes.len(),
            );
            buf.set_len(buf_len + self.token.len() + options_bytes.len());
        }

        if self.get_code() != MessageClass::Empty && !self.payload.is_empty() {
            let _ = buf.push(0xFF);
            unsafe {
                use core::ptr;
                let buf_len = buf.len();
                ptr::copy(
                    self.payload.as_ptr(),
                    buf.as_mut_ptr().add(buf.len()),
                    self.payload.len(),
                );
                buf.set_len(buf_len + self.payload.len());
            }
        }
        Ok(buf)
    }

    fn to_uint<T>(&self, encoded: &[u8]) -> Result<T, IncompatibleOptionValueFormat>
    where
        T: TryFrom<u64>
            + From<u8>
            + core::ops::Shl<usize, Output = T>
            + core::ops::Add<Output = T>
            + Default,
    {
        let value_size = size_of::<T>();
        if encoded.len() > value_size {
            let mut s = String::<50>::new();
            match write!(
                s,
                "overflow: got {} bytes, expected {}",
                encoded.len(),
                value_size
            ) {
                Err(_) => return Err(IncompatibleOptionValueFormat { message: s }),
                _ => {}
            }
        }
        Ok(encoded
            .iter()
            .fold(T::default(), |acc, &b| (acc << 8) + T::from(b)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_header_codes() {
        for code in 0..255 {
            let p = Packet::new(
                MessageType::Confirmable,
                code.into(),
                /* version= */ 1,
                /* message_id= */ 1,
                /* token= */ &[],
                /* options= */ &mut Vec::new(),
                /* payload= */ &[],
            );
            let class: MessageClass = code.into();
            // valid items
            if !matches!(class, MessageClass::Reserved(_)) {
                assert_eq!(u8::from(class), code);
                assert_eq!(class, p.get_code());
            }
        }
    }

    #[test]
    fn from_bytes_fail() {
        let b: &[u8] = &[1, 2, 3];
        let p = Packet::from_bytes(b);
        assert_eq!(MessageError::InvalidPacketLength, p.unwrap_err());
    }

    #[test]
    fn types() {
        let p_acked = Packet::new(
            MessageType::Acknowledgement,
            MessageClass::Request(RequestType::Get),
            /* version= */ 1,
            /* message_id= */ 1,
            /* token= */ &[],
            /* options= */ &mut Vec::new(),
            /* payload= */ &[],
        );
        assert_eq!(MessageType::Acknowledgement, p_acked.get_type());
        let p_confirmed = Packet::new(
            MessageType::Confirmable,
            MessageClass::Request(RequestType::Get),
            /* version= */ 1,
            /* message_id= */ 1,
            /* token= */ &[],
            /* options= */ &mut Vec::new(),
            /* payload= */ &[],
        );
        assert_eq!(MessageType::Confirmable, p_confirmed.get_type());
        let p_notconfirmed = Packet::new(
            MessageType::NonConfirmable,
            MessageClass::Request(RequestType::Get),
            /* version= */ 1,
            /* message_id= */ 1,
            /* token= */ &[],
            /* options= */ &mut Vec::new(),
            /* payload= */ &[],
        );
        assert_eq!(MessageType::NonConfirmable, p_notconfirmed.get_type());
        let p_reset = Packet::new(
            MessageType::Reset,
            MessageClass::Request(RequestType::Get),
            /* version= */ 1,
            /* message_id= */ 1,
            /* token= */ &[],
            /* options= */ &mut Vec::new(),
            /* payload= */ &[],
        );
        assert_eq!(MessageType::Reset, p_reset.get_type());
    }

    #[test]
    fn test_decode_packet_with_options() {
        let buf = [
            0x44, 0x01, 0x84, 0x9e, 0x51, 0x55, 0x77, 0xe8, 0xb2, 0x48, 0x69, 0x04, 0x54, 0x65,
            0x73, 0x74, 0x43, 0x61, 0x3d, 0x31,
        ];
        let packet = Packet::from_bytes(&buf);
        assert!(packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(packet.get_version(), 1);
        assert_eq!(packet.get_type(), MessageType::Confirmable);
        assert_eq!(packet.get_token_length(), 4);
        assert_eq!(packet.get_code(), MessageClass::Request(RequestType::Get));
        assert_eq!(packet.get_message_id(), 33950);
        assert_eq!(packet.get_token(), &[0x51, 0x55, 0x77, 0xE8]);
        assert_eq!(packet.options.len(), 3);

        let mut uri_path_iter = packet.get_options(CoapOption::UriPath);
        let patch_part1 = uri_path_iter.next();
        assert!(patch_part1.is_some());
        assert_eq!(patch_part1.unwrap().value, "Hi".as_bytes());
        let patch_part2 = uri_path_iter.next();
        assert!(patch_part2.is_some());
        assert_eq!(patch_part2.unwrap().value, "Test".as_bytes());
        assert!(uri_path_iter.next().is_none());

        let mut uri_query_iter = packet.get_options(CoapOption::UriQuery);
        let uri_query_item = uri_query_iter.next();
        assert!(uri_query_item.is_some());
        assert_eq!(uri_query_item.unwrap().value, "a=1".as_bytes());
        assert!(uri_query_iter.next().is_none());
    }

    #[test]
    fn test_decode_packet_with_payload() {
        let buf = [
            0x64, 0x45, 0x13, 0xFD, 0xD0, 0xE2, 0x4D, 0xAC, 0xFF, 0x48, 0x65, 0x6C, 0x6C, 0x6F,
        ];
        let packet = Packet::from_bytes(&buf);
        assert!(packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(packet.get_version(), 1);
        assert_eq!(packet.get_type(), MessageType::Acknowledgement);
        assert_eq!(packet.get_token_length(), 4);
        assert_eq!(
            packet.get_code(),
            MessageClass::Response(ResponseType::Content)
        );
        assert_eq!(packet.message_id, 5117);
        assert_eq!(packet.get_token(), &[0xD0, 0xE2, 0x4D, 0xAC]);
        assert_eq!(packet.payload, "Hello".as_bytes().to_vec());
    }

    #[test]
    fn test_encode_packet_with_options() {
        let options = &[
            OptionPair {
                num: CoapOption::UriPath.into(),
                value: "Hi".as_bytes(),
            },
            OptionPair {
                num: CoapOption::UriPath.into(),
                value: "Test".as_bytes(),
            },
            OptionPair {
                num: CoapOption::UriQuery.into(),
                value: "a=1".as_bytes(),
            },
        ];
        let packet = Packet::new(
            MessageType::Confirmable,
            MessageClass::Request(RequestType::Get),
            /* version= */ 1,
            /* message_id= */ 33950,
            /* token= */ &[0x51, 0x55, 0x77, 0xE8],
            /* options= */ &mut Vec::from_slice(options).unwrap(),
            /* payload= */ &[],
        );
        assert_eq!(
            packet.to_bytes().unwrap(),
            &[
                0x44, 0x01, 0x84, 0x9e, 0x51, 0x55, 0x77, 0xe8, 0xb2, 0x48, 0x69, 0x04, 0x54, 0x65,
                0x73, 0x74, 0x43, 0x61, 0x3d, 0x31
            ]
        );
    }

    #[test]
    fn test_encode_packet_with_payload() {
        let packet = Packet::new(
            MessageType::Acknowledgement,
            MessageClass::Response(ResponseType::Content),
            /* version= */ 1,
            /* message_id= */ 5117,
            /* token= */ &[0xD0, 0xE2, 0x4D, 0xAC],
            /* options= */ &mut Vec::new(),
            /* payload= */ "Hello".as_bytes(),
        );
        assert_eq!(
            packet.to_bytes().unwrap(),
            &[0x64, 0x45, 0x13, 0xFD, 0xD0, 0xE2, 0x4D, 0xAC, 0xFF, 0x48, 0x65, 0x6C, 0x6C, 0x6F]
        );
    }

    #[test]
    fn test_encode_decode_content_format() {
        let options = &[OptionPair {
            num: CoapOption::ContentFormat.into(),
            value: &u16::try_from(usize::from(ContentFormat::TextPlain))
                .unwrap()
                .to_be_bytes(),
        }];
        let packet = Packet::new(
            MessageType::NonConfirmable,
            MessageClass::Request(RequestType::Get),
            /* version= */ 1,
            /* message_id= */ 5117,
            /* token= */ &[0xD0, 0xE2, 0x4D, 0xAC],
            /* options= */ &mut Vec::from_slice(options).unwrap(),
            /* payload= */ "Hello".as_bytes(),
        );
        assert_eq!(
            ContentFormat::TextPlain,
            ContentFormat::try_from(packet.get_content_format_value().unwrap() as usize)
                .ok()
                .unwrap()
        );
    }

    #[test]
    fn test_encode_decode_content_format_without_msb() {
        let options = &[OptionPair {
            num: CoapOption::ContentFormat.into(),
            value: &u16::try_from(usize::from(ContentFormat::ApplicationJSON))
                .unwrap()
                .to_be_bytes(),
        }];
        let packet = Packet::new(
            MessageType::NonConfirmable,
            MessageClass::Request(RequestType::Get),
            /* version= */ 1,
            /* message_id= */ 5117,
            /* token= */ &[0xD0, 0xE2, 0x4D, 0xAC],
            /* options= */ &mut Vec::from_slice(options).unwrap(),
            /* payload= */ "Hello".as_bytes(),
        );
        assert_eq!(
            ContentFormat::ApplicationJSON,
            ContentFormat::try_from(packet.get_content_format_value().unwrap() as usize)
                .ok()
                .unwrap()
        );
    }

    #[test]
    fn test_encode_decode_content_format_with_msb() {
        let options = &[OptionPair {
            num: CoapOption::ContentFormat.into(),
            value: &u16::try_from(usize::from(ContentFormat::ApplicationSensmlXML))
                .unwrap()
                .to_be_bytes(),
        }];
        let packet = Packet::new(
            MessageType::NonConfirmable,
            MessageClass::Request(RequestType::Get),
            /* version= */ 1,
            /* message_id= */ 5117,
            /* token= */ &[0xD0, 0xE2, 0x4D, 0xAC],
            /* options= */ &mut Vec::from_slice(options).unwrap(),
            /* payload= */ "Hello".as_bytes(),
        );
        assert_eq!(
            ContentFormat::ApplicationSensmlXML,
            ContentFormat::try_from(packet.get_content_format_value().unwrap() as usize)
                .ok()
                .unwrap()
        );
    }

    #[test]
    fn test_decode_empty_content_format() {
        let packet = Packet::new(
            MessageType::NonConfirmable,
            MessageClass::Request(RequestType::Get),
            /* version= */ 1,
            /* message_id= */ 5117,
            /* token= */ &[0xD0, 0xE2, 0x4D, 0xAC],
            /* options= */ &mut Vec::new(),
            /* payload= */ "Hello".as_bytes(),
        );
        assert!(packet.get_content_format_value().is_none());
    }

    #[test]
    fn option() {
        for i in 0..512 {
            assert_eq!(i, CoapOption::from(i).into());
        }
    }

    #[test]
    fn content_format() {
        for i in 0..512 {
            if let Ok(o) = ContentFormat::try_from(i) {
                assert_eq!(i, o.into());
            }
        }
    }

    #[test]
    fn observe_option() {
        for i in 0..8 {
            if let Ok(o) = ObserveOption::try_from(i) {
                assert_eq!(i, o.into());
            }
        }
    }

    #[test]
    fn options() {
        let options = &[
            OptionPair {
                num: CoapOption::UriHost.into(),
                value: &[0],
            },
            OptionPair {
                num: CoapOption::UriPath.into(),
                value: &[1],
            },
            OptionPair {
                num: CoapOption::ETag.into(),
                value: &[2],
            },
        ];
        let packet = Packet::new(
            MessageType::NonConfirmable,
            MessageClass::Request(RequestType::Get),
            /* version= */ 1,
            /* message_id= */ 5117,
            /* token= */ &[0xD0, 0xE2, 0x4D, 0xAC],
            /* options= */ &mut Vec::from_slice(options).unwrap(),
            /* payload= */ "Hello".as_bytes(),
        );
        assert_eq!(3, packet.options().len());

        let bytes = packet.to_bytes().unwrap();
        let pp = Packet::from_bytes(&bytes).unwrap();
        assert_eq!(3, pp.options().len());
    }

    // #[test]
    // fn test_option_u32_format() {
    //     let options = &[
    //         OptionPair {
    //             num: CoapOption::Observe.into(),
    //             value: [],
    //         },
    //         OptionPair {
    //             num: CoapOption::Observe.into(),
    //             value: &[1],
    //         },
    //         OptionPair {
    //             num: CoapOption::Observe.into(),
    //             value: &[2],
    //         },
    //     ];
    //     let packet = Packet::new(
    //         MessageType::NonConfirmable,
    //         MessageClass::Request(RequestType::Get),
    //         /* version= */ 1,
    //         /* message_id= */ 5117,
    //         /* token= */ &[0xD0, 0xE2, 0x4D, 0xAC],
    //         /* options= */ &mut Vec::from_slice(options).unwrap(),
    //         /* payload= */ "Hello".as_bytes(),
    //     );
    //     let values = vec![0, 100, 1000, 10000, u32::MAX];
    //     let expected = values.iter().map(|&x| Ok(OptionValueU32(x))).collect();
    //     assert_eq!(actual, Some(expected));
    // }

    // #[test]
    // fn test_option_utf8_format() {
    //     let mut p = Packet::new();
    //     let option_key = CoapOption::UriPath;
    //     let values = vec!["", "simple", "unicode 😁 stuff"];
    //     for &value in &values {
    //         p.add_option_as(option_key, OptionValueString(value.to_owned()));
    //     }
    //     let expected = values
    //         .iter()
    //         .map(|&x| Ok(OptionValueString(x.to_owned())))
    //         .collect();
    //     let actual = p.get_options_as::<OptionValueString>(option_key);
    //     assert_eq!(actual, Some(expected));
    // }

    #[test]
    fn observe_none() {
        let packet = Packet::new(
            MessageType::NonConfirmable,
            MessageClass::Request(RequestType::Get),
            /* version= */ 1,
            /* message_id= */ 5117,
            /* token= */ &[0xD0, 0xE2, 0x4D, 0xAC],
            /* options= */ &mut Vec::new(),
            /* payload= */ "Hello".as_bytes(),
        );
        assert_eq!(None, packet.get_observe_value());
        // p.set_observe_value(0);
        // assert_eq!(Some(Ok(0)), p.get_observe_value());
    }

    #[test]
    fn observe_some() {
        let options = &[OptionPair {
            num: CoapOption::Observe.into(),
            value: &[10],
        }];
        let packet = Packet::new(
            MessageType::NonConfirmable,
            MessageClass::Request(RequestType::Get),
            /* version= */ 1,
            /* message_id= */ 5117,
            /* token= */ &[0xD0, 0xE2, 0x4D, 0xAC],
            /* options= */ &mut Vec::from_slice(options).unwrap(),
            /* payload= */ "Hello".as_bytes(),
        );
        assert_eq!(Some(10), packet.get_observe_value());
    }

    #[test]
    fn options_limit_exceeded() {
        let buf = [
            0x40, 0x01, 0x00, 0x00, 0x61, 0x31, 0x01, 0x32, 0x01, 0x33, 0x01, 0x34, 0x01, 0x35,
            0x01, 0x36, 0x01, 0x37, 0x01, 0x38, 0x01, 0x39, 0x02, 0x31, 0x30, 0x02, 0x31, 0x31,
            0x02, 0x31, 0x32, 0x02, 0x31, 0x33, 0x02, 0x31, 0x34, 0x02, 0x31, 0x35, 0x02, 0x31,
            0x36, 0x02, 0x31, 0x37, 0x02, 0x31, 0x38, 0x02, 0x31, 0x39, 0x02, 0x32, 0x30, 0x02,
            0x32, 0x31, 0x02, 0x32, 0x32, 0x02, 0x32, 0x33, 0x02, 0x32, 0x34, 0x02, 0x32, 0x35,
            0x02, 0x32, 0x36, 0x02, 0x32, 0x37, 0x02, 0x32, 0x38, 0x02, 0x32, 0x39, 0x02, 0x33,
            0x30, 0x02, 0x33, 0x31, 0x02, 0x33, 0x32, 0x02, 0x33, 0x33,
        ];
        let p = Packet::from_bytes(&buf);
        assert_eq!(MessageError::OptionsLimitExceeded, p.unwrap_err());
    }
}
