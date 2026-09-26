# Socks7 / V2rei

**Next-generation ultra-lightweight, high-performance proxy protocol (Version `0x07`)**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/Status-Active%20Development-yellow.svg)]()

**Dual branding**: Use it as **Socks7** or **V2rei** — both are official.

- Protocol Version: `0x07`
- Website: [v2rei.surf](https://v2rei.surf)
- GitHub: [socks7-v2rei/socks7](https://github.com/socks7-v2rei/socks7)

---

## Overview

Socks7 (also known as V2rei) is a clean, modern proxy protocol designed from the ground up for:

- Extreme performance
- Minimal binary overhead
- Native 0-RTT support
- High concurrency with low memory usage
- Long-term stability

Built in **Rust** for the v2rei network.

---

## Features

- Extremely low protocol overhead
- Native 0-RTT via Initial Data
- Clean binary framing
- Extensible Options system
- Memory-safe implementation
- Full type-safe protocol definitions
- Dual naming: **Socks7** and **V2rei**

---

## Current Status

| Component                | Status          |
|--------------------------|-----------------|
| Core Protocol Definitions| ✅ Complete     |
| CONNECT command          | ✅ Working      |
| BIND / UDP ASSOCIATE     | 🚧 Planned      |
| Authentication           | 🚧 Planned      |
| Multiplexing             | 🚧 Planned      |
| Client library           | ✅ Basic        |
| Server                   | ✅ Basic        |

---

## Quick Start

### Build

```bash
cargo build --release
```

### Run Server

```bash
./target/release/socks7 server --listen 0.0.0.0:1080
```

With logs:

```bash
RUST_LOG=debug ./target/release/socks7 server
```

---

## Protocol Highlights

- Version byte: `0x07`
- Minimal request / reply structure
- Support for IPv4, IPv6 and Domain
- Initial Data field for true 0-RTT behavior
- Typed Options (auth, padding, multiplexing, vendor extensions)
- Clear error codes

---

## Project Structure

```
src/
├── protocol/          # Core protocol definitions
│   ├── address.rs
│   ├── command.rs
│   ├── error.rs
│   ├── message.rs
│   ├── option.rs
│   └── reply.rs
├── server/            # Server implementation
├── client/            # Client library
├── lib.rs
└── main.rs
```

---

## License

MIT License — see [LICENSE](LICENSE)

---

**Socks7** = **V2rei**  
Lightweight by design. Powerful by nature.
