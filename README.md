# Socks7 / V2rei

**Production SOCKS5 Proxy + Desktop Client**

Dual branding: **Socks7** = **V2rei**

Repository: https://github.com/socks7-v2rei/socks7

---

## 1. Server (VPS / Cloud)

### One-click Docker

```bash
git clone https://github.com/socks7-v2rei/socks7.git
cd socks7
docker build -t socks7-v2rei .
docker run -d --name socks7 -p 7777:7777 --restart unless-stopped socks7-v2rei
docker logs socks7
```

You will see:

```
Username : ........
Password : ........
Listen   : 0.0.0.0:7777
```

### docker-compose

```bash
docker compose up -d --build
docker compose logs -f
```

---

## 2. Desktop Client (Windows / Linux / macOS)

### Build from source

```bash
git clone https://github.com/socks7-v2rei/socks7.git
cd socks7
cargo build --release
```

### Run client

```bash
./target/release/socks7 client \
  --upstream YOUR_SERVER_IP:7777 \
  --username YOUR_USER \
  --password YOUR_PASS
```

Then set system / browser proxy:

```
SOCKS5 → 127.0.0.1:1080
```

### Windows

```powershell
cargo build --release
.\target\release\socks7.exe client --upstream IP:7777 --username USER --password PASS
```

---

## 3. Mobile (Android / iOS)

Use any SOCKS5 app:

| Platform | Recommended Apps              |
|----------|-------------------------------|
| Android  | V2Box, NekoBox, SocksDroid    |
| iOS      | Shadowrocket, Streisand, Quantumult X |

```
Type     : SOCKS5
Server   : YOUR_SERVER_IP
Port     : 7777
Username : (from docker logs)
Password : (from docker logs)
```

### Full link format

```
socks5://USERNAME:PASSWORD@SERVER_IP:7777
```

---

## 4. Commands

```bash
# Server
socks7 server --listen 0.0.0.0:7777 --username user --password pass

# Desktop Client
socks7 client --upstream 1.2.3.4:7777 --username user --password pass
```

---

## 5. Features

- Full SOCKS5 (CONNECT)
- Username / Password authentication
- Auto random credentials in Docker
- Desktop client (local → remote)
- Docker + docker-compose
- Cross-platform (Linux / Windows / macOS)
- Dual branding: Socks7 / V2rei
- Port 7777 (dedicated)

---

## 6. Project Structure

```
src/
├── server/     # SOCKS5 server
├── bridge/     # Desktop client (local forwarder)
├── client/     # Protocol client helpers
├── auth/       # Authentication
├── protocol/   # Protocol definitions
└── main.rs     # CLI
```

---

## License

MIT License

**Socks7** = **V2rei**  
Lightweight. Stable. Ready.
