# coap-tiny
`coap-tiny` is a minimalistic Constrained Application Protocol (CoAP) implementation for embedded systems, written in Rust. This library provides a lightweight CoAP stack designed for devices with limited resources, making it suitable for IoT and embedded systems applications.

## Features

- Lightweight CoAP implementation
- Minimal resource usage, ideal for constrained devices
- Written in Rust, ensuring memory safety and performance
- Suitable for low-power devices

## Installation

To add coap-tiny to your project, add the following line to your `Cargo.toml`:
```
[dependencies]
coap-tiny = "0.1"
```

## Usage
### Examples
```
use coap_tiny::packet::{MessageClass, MessageType, Packet};
use heapless::Vec;

fn main() {
    let buf = [
        0x64, 0x45, 0x13, 0xFD, 0xD0, 0xE2, 0x4D, 0xAC, 0xFF, 0x48, 0x65, 0x6C, 0x6C, 0x6F,
    ];
    let packet = Packet::from_bytes(&buf).unwrap();
    packet.get_code();
    packet.get_message_id();
    packet.get_payload();
    packet.get_token();
    packet.get_type();

    let response_packet = Packet::new(
        MessageType::Acknowledgement,
        MessageClass::Response(coap_tiny::packet::ResponseType::Created),
        packet.get_version(),
        packet.get_message_id(),
        packet.get_token(),
        &mut Vec::new(),
        b"response".as_slice(),
    );
    response_packet.to_bytes().unwrap();
}
```

### Configurable Constants
The following constants can be adjusted in `src/lib.rs` based on your project’s needs:
```
pub const PACKET_MAX_SIZE: usize = 4096; // Maximum size of CoAP packets
pub const MAX_OPTIONS: usize = 32;       // Maximum number of CoAP options
pub const PATH_MAX_SIZE: usize = 128;    // Maximum length of resource paths
```

## Contributing
Contributions are welcome! Please open issues or submit pull requests.

## License

This project is licensed under the MIT License.
