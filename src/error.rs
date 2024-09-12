use core::{fmt, num::TryFromIntError};

use heapless::String;

/// The errors that can occur when encoding/decoding packets.
#[derive(Debug, PartialEq)]
pub enum MessageError {
    InvalidHeader,
    InvalidPacketLength,
    InvalidTokenLength,
    InvalidOptionDelta,
    InvalidOptionLength,
    InvalidOption,
}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MessageError::InvalidHeader => {
                write!(f, "CoAP error: invalid header")
            }
            MessageError::InvalidPacketLength => {
                write!(
                    f,
                    "CoAP error: invalid packet length, consider using BlockHandler"
                )
            }
            MessageError::InvalidTokenLength => {
                write!(f, "CoAP error: invalid token length")
            }
            MessageError::InvalidOptionDelta => {
                write!(f, "CoAP error: invalid option delta")
            }
            MessageError::InvalidOptionLength => {
                write!(f, "CoAP error: invalid option length")
            }
            MessageError::InvalidOption => {
                write!(f, "CoAP error: invalid option")
            }
        }
    }
}

/// The error that can occur when parsing a content-format.
#[derive(Debug, PartialEq)]
pub struct InvalidContentFormat;

impl fmt::Display for InvalidContentFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CoAP error: invalid content-format number")
    }
}

/// The error that can occur when parsing an observe option value.
#[derive(Debug, PartialEq)]
pub struct InvalidObserve;

impl fmt::Display for InvalidObserve {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CoAP error: invalid observe option number")
    }
}

/// The error that can occur when parsing an option value.
#[derive(Debug, PartialEq)]
pub struct IncompatibleOptionValueFormat {
    pub message: String<50>,
}

impl fmt::Display for IncompatibleOptionValueFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Incompatible option value: {}", self.message)
    }
}

/// The errors that can occur when constructing a new block value.
#[derive(Debug, PartialEq)]
pub enum InvalidBlockValue {
    SizeExponentEncodingError(usize),
    TypeBoundsError(TryFromIntError),
}

impl fmt::Display for InvalidBlockValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidBlockValue::SizeExponentEncodingError(size) => {
                write!(f, "size cannot be encoded {}", size)
            }
            InvalidBlockValue::TypeBoundsError(err) => {
                write!(f, "size provided is outside type bounds: {}", err)
            }
        }
    }
}
