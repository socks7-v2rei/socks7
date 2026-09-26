# Socks7 / V2rei

**Next-generation ultra-lightweight, high-performance proxy protocol (Version `0x07`)**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

**Dual branding**: Officially usable as **Socks7** or **V2rei**.

- Protocol Version: `0x07`
- Website: [v2rei.surf](https://v2rei.surf)
- Repository: [socks7-v2rei/socks7](https://github.com/socks7-v2rei/socks7)

---

## Features

- Extremely low protocol overhead
- Native 0-RTT support via Initial Data
- Full CONNECT command (production-ready)
- UDP ASSOCIATE support
- Optional Username/Password authentication
- Clean binary framing + extensible Options
- Timeouts, nodelay, connection hardening
- Memory-safe Rust implementation
- Dual naming: **Socks7** = **V2rei**

---

## Quick Start

### Build

```bash
cargo build --release
```

### Run Server (No Authentication)

```bash
./target/release/socks7 server --listen 0.0.0.0:1080
```

### Run Server with Authentication

```bash
./target/release/socks7 server \
  --listen 0.0.0.0:1080 \
  --username myuser \
  --password mypass
```

### Environment

```bash
RUST_LOG=debug ./target/release/socks7 server
```

---

## Protocol Overview

| Item                    | Value                          |
|-------------------------|--------------------------------|
| Version                 | `0x07`                         |
| Commands                | CONNECT, UDP ASSOCIATE, NOOP, BIND |
| Address Types           | IPv4, IPv6, Domain             |
| Authentication          | NoAuth + Username/Password     |
| 0-RTT                   | Yes (Initial Data)             |
| Options                 | Fully extensible               |

---

## Project Structure

```
src/
├── protocol/     # Core protocol (address, command, message, option, reply)
├── server/       # High-performance server
├── client/       # Client library
├── auth/         # Authentication
├── lib.rs
└── main.rs
```

---

## Status

| Feature                 | Status     |
|-------------------------|------------|
| Protocol definitions    | Complete   |
| CONNECT                 | Complete   |
| UDP ASSOCIATE           | Complete   |
| Authentication          | Complete   |
| Timeouts & Hardening    | Complete   |
| BIND                    | Basic      |
| Multiplexing            | Planned    |

---

## License

MIT License

---

**Socks7** = **V2rei**  
Lightweight by design. Powerful by nature.  
Built for the v2rei network.
