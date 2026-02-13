#!/bin/sh
set -e

# Check if the first argument is "web"
if [ "$1" = "web" ]; then
    echo "Starting Caddy web server..."
    exec caddy run --config /etc/caddy/Caddyfile --adapter caddyfile
else
    # Default: run baza CLI with all arguments
    exec /bin/baza "$@"
fi
