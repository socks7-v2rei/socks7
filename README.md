# Socks7 / V2rei

**Lightweight SOCKS5 Proxy + Desktop Client**

Dual branding: **Socks7** = **V2rei**

---

## Server (already running on your VPS)

```bash
# Docker
docker run -d --name socks7 -p 7777:7777 --restart unless-stopped socks7-v2rei
docker logs socks7
```

---

## Desktop Client (Windows / Linux / macOS)

### 1. Build

```bash
git clone https://github.com/socks7-v2rei/socks7.git
cd socks7
cargo build --release
```

### 2. Run Client

```bash
./target/release/socks7 client \
  --upstream 193.233.218.5:7777 \
  --username YOUR_USERNAME \
  --password YOUR_PASSWORD
```

### 3. Set system / browser proxy to:

```
SOCKS5 → 127.0.0.1:1080
```

Now all traffic goes through your server.

---

## Mobile

Use any SOCKS5 client app:

- **Android**: V2Box, NekoBox, SocksDroid, ...
- **iOS**: Shadowrocket, Quantumult X, Streisand, ...

```
Type     : SOCKS5
Server   : 193.233.218.5
Port     : 7777
Username : (from docker logs)
Password : (from docker logs)
```

---

## Features

- Full SOCKS5 support
- Username / Password authentication
- Desktop Client (local → remote)
- Docker one-click server
- Dual branding Socks7 / V2rei

---

## License

MIT
