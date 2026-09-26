#!/bin/bash
set -e

if [ -z "$SOCKS7_USERNAME" ]; then
    SOCKS7_USERNAME=$(cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w 12 | head -n 1)
fi

if [ -z "$SOCKS7_PASSWORD" ]; then
    SOCKS7_PASSWORD=$(cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w 16 | head -n 1)
fi

echo "============================================================"
echo "  Socks7 / V2rei Proxy is starting..."
echo "============================================================"
echo ""
echo "  Protocol         : SOCKS5 (compatible)"
echo "  Listen Address   : 0.0.0.0:7777"
echo ""
echo "  Username         : $SOCKS7_USERNAME"
echo "  Password         : $SOCKS7_PASSWORD"
echo ""
echo "  Dual branding    : Socks7  |  V2rei"
echo "  Website          : https://v2rei.surf"
echo "============================================================"
echo ""
echo "  Now usable directly from Mobile (Telegram, Browser, etc)"
echo "============================================================"
echo ""

exec socks7 server \
    --listen 0.0.0.0:7777 \
    --username "$SOCKS7_USERNAME" \
    --password "$SOCKS7_PASSWORD"
