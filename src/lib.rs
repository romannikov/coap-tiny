#![no_std]

pub mod error;
pub mod packet;
pub mod request;
pub mod response;

pub const PACKET_MAX_SIZE: usize = 3000;
pub const PACKET_PAYLOAD_MAX_SIZE: usize = 2000;
pub const MAX_OPTIONS: usize = 32;
pub const PATH_MAX_SIZE: usize = 100;
