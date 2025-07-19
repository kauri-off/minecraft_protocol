# minecraft_protocol

`minecraft_protocol` is a Rust library for working with Minecraft's network protocol. It provides tools for serialization, deserialization, packet handling, and encryption streams.

## ✨ Features

- **Packet Serialization/Deserialization**
  - Traits for encoding/decoding Minecraft packets.
  - Procedural macro `#[derive(Packet)]` for automatically generating implementations.
- **VarInt and Numeric Utilities**
  - Includes helpers for working with Minecraft's VarInt format and byte encoding for numeric types.
- **CFB8 Encryption Streams**
  - Async read/write wrappers over `TcpStream` for AES-128-CFB8 encryption as used in Minecraft protocol.

## 🛠 Dependencies

- [`openssl`](https://crates.io/crates/openssl): encryption (AES-128-CFB8).
- [`tokio`](https://crates.io/crates/tokio): async I/O support.
- [`thiserror`](https://crates.io/crates/thiserror): error handling.
- [`syn`, `quote`, `proc-macro2`](https://doc.rust-lang.org/proc_macro/): for procedural macros in `minecraft_protocol_derive`.

## 📥 Usage

Add this crate to your project:

```toml
[dependencies]
minecraft_protocol = { git = "https://github.com/kauri-off/minecraft_protocol.git" }
```

Example of using the derive macro:

```rust
use minecraft_protocol::{Packet, varint::VarInt};

#[derive(Packet)]
#[packet(0x00)]
struct Handshake {
    protocol_version: VarInt,
    server_address: String,
    server_port: u16,
    next_state: VarInt,
}
```

## 🔒 AES-128-CFB8 Stream

Create an encrypted stream from a `TcpStream`:

```rust
use minecraft_protocol::cfb8_stream::CFB8Stream;
use tokio::net::TcpStream;

let tcp_stream = TcpStream::connect("example.com:25565").await?;
let key: [u8; 16] = [0; 16]; // replace with actual shared secret

let encrypted_stream = CFB8Stream::new_from_tcp(tcp_stream, &key)?;
```
