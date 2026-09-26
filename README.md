# Socks7 / V2rei

**Next-generation ultra-lightweight, high-performance proxy protocol (Version `0x07`)**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Docker](https://img.shields.io/badge/Docker-Ready-blue.svg)]()

**Dual branding**: Officially usable as **Socks7** or **V2rei**.

- Protocol Version: `0x07`
- Website: [v2rei.surf](https://v2rei.surf)
- Repository: [socks7-v2rei/socks7](https://github.com/socks7-v2rei/socks7)

---

## Features

- Extremely low protocol overhead
- Native 0-RTT support via Initial Data
- Full CONNECT + UDP ASSOCIATE
- Automatic random Username / Password generation (Docker)
- Optional Username/Password authentication
- Timeouts, hardening, production-ready
- Dual naming: **Socks7** = **V2rei**

---

## Quick Start with Docker (Recommended)

### 1. Build & Run (auto-generates random credentials)

```bash
docker build -t socks7-v2rei .
docker run -d --name socks7 -p 1080:1080 socks7-v2rei
```

### 2. See the generated username & password

```bash
docker logs socks7
```

You will see something like:

```
============================================================
  Socks7 / V2rei Proxy is starting...
============================================================

  Protocol Version : 0x07
  Listen Address   : 0.0.0.0:1080

  Username         : aB3xK9mP2qR7
  Password         : zY8nQ4wE1tU6vC9s

  Dual branding    : Socks7  |  V2rei
  Website          : https://v2rei.surf
============================================================
```

### 3. Using docker-compose

```bash
docker-compose up -d
docker-compose logs -f
```

### Custom credentials (optional)

```bash
docker run -d --name socks7 -p 1080:1080 \
  -e SOCKS7_USERNAME=myuser \
  -e SOCKS7_PASSWORD=mypass \
  socks7-v2rei
```

---

## Build from Source

```bash
cargo build --release
./target/release/socks7 server --listen 0.0.0.0:1080
```

With authentication:

```bash
./target/release/socks7 server \
  --listen 0.0.0.0:1080 \
  --username myuser \
  --password mypass
```

---

## Protocol Status

| Feature                 | Status     |
|-------------------------|------------|
| Protocol definitions    | Complete   |
| CONNECT                 | Complete   |
| UDP ASSOCIATE           | Complete   |
| Authentication          | Complete   |
| Docker + Auto Creds     | Complete   |
| Timeouts & Hardening    | Complete   |
| BIND                    | Basic      |
| Multiplexing            | Planned    |

---

## Project Structure

```
├── Dockerfile
├── docker-entrypoint.sh
├── docker-compose.yml
├── src/
│   ├── protocol/     # Core protocol
│   ├── server/       # Production server
│   ├── client/       # Client library
│   ├── auth/         # Authentication
│   ├── lib.rs
│   └── main.rs
└── README.md
```

---

## License

MIT License

---

**Socks7** = **V2rei**  
Lightweight by design. Powerful by nature.  
Built for the v2rei network.
