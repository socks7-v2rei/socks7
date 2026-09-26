# Socks7 / V2rei

**Next-generation ultra-lightweight, high-performance proxy protocol (Version `0x07`)**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Docker](https://img.shields.io/badge/Docker-Ready-blue.svg)]()

**Dual branding**: Officially usable as **Socks7** or **V2rei**.

---

## How to use (Practical for everyone)

### Option A: Docker (Server side)

```bash
git clone https://github.com/socks7-v2rei/socks7.git
cd socks7
docker build -t socks7-v2rei .
docker run -d --name socks7 -p 1080:1080 socks7-v2rei
docker logs socks7          # shows random username & password
```

### Option B: Local SOCKS5 Bridge (Client side - Recommended)

This makes the proxy work with **any** normal application (Browser, Telegram, curl, etc).

```bash
# Build
cargo build --release

# Run bridge (local SOCKS5 → remote Socks7)
./target/release/socks7 bridge \
  --listen 127.0.0.1:1080 \
  --upstream YOUR_SERVER_IP:1080
```

Now set your system / browser / Telegram SOCKS5 proxy to:

```
127.0.0.1:1080
```

Done. Everything works.

---

## Full Commands

### 1. Run pure Socks7 server

```bash
./target/release/socks7 server --listen 0.0.0.0:1080
```

With auth:
```bash
./target/release/socks7 server --listen 0.0.0.0:1080 \
  --username myuser --password mypass
```

### 2. Run SOCKS5 Bridge (for normal apps)

```bash
./target/release/socks7 bridge \
  --listen 127.0.0.1:1080 \
  --upstream 1.2.3.4:1080
```

---

## Features

- Full CONNECT + UDP ASSOCIATE
- Automatic random credentials in Docker
- **SOCKS5 Bridge** → usable by any application
- Authentication support
- Dual branding: Socks7 = V2rei
- Production hardening + timeouts

---

## Project Structure

```
src/
├── protocol/     # Core Socks7 protocol
├── server/       # Socks7 server
├── client/       # Socks7 client
├── bridge/       # SOCKS5 → Socks7 bridge
├── auth/
└── main.rs
```

---

## License

MIT License

**Socks7** = **V2rei**  
Lightweight by design. Powerful by nature.
