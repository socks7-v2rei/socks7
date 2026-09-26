#!/bin/bash
set -e

# Generate random credentials if not provided via environment
if [ -z "$SOCKS7_USERNAME" ]; then
    SOCKS7_USERNAME=$(cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w 12 | head -n 1)
fi

if [ -z "$SOCKS7_PASSWORD" ]; then
    SOCKS7_PASSWORD=$(cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w 16 | head -n 1)
fi

# Print connection info clearly
echo "============================================================"
echo "  Socks7 / V2rei Proxy is starting..."
echo "============================================================"
echo ""
echo "  Protocol Version : 0x07"
echo "  Listen Address   : 0.0.0.0:1080"
echo ""
echo "  Username         : $SOCKS7_USERNAME"
echo "  Password         : $SOCKS7_PASSWORD"
echo ""
echo "  Dual branding    : Socks7  |  V2rei"
echo "  Website          : https://v2rei.surf"
echo "============================================================"
echo ""

# Export so the binary can use them
export SOCKS7_USERNAME
export SOCKS7_PASSWORD

# Run the server with generated credentials
exec socks7 server \
    --listen 0.0.0.0:1080 \
    --username "$SOCKS7_USERNAME" \
    --password "$SOCKS7_PASSWORD" \
    "$@"
