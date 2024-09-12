#![no_std]

pub mod error;
pub mod packet;
pub mod request;
pub mod response;

pub const PACKET_MAX_SIZE: usize = 4096;
pub const MAX_OPTIONS: usize = 32;
pub const PATH_MAX_SIZE: usize = 128;
