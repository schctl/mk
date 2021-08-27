#!/usr/bin/env sh

# Build and run `mk` with expected permissions.
# (Disclaimer: I don't know how to shell script)

set -e

# Use `mk` if it is in the path ( ͡° ͜ʖ ͡°)-b
if [ $(which ./mk 2>/dev/null) ]; then
    priv=./mk
elif [ $(which mk 2>/dev/null) ]; then
    priv=mk
elif [ $(which doas 2>/dev/null) ]; then
    priv=doas
else
    priv=sudo
fi

mode=debug
bin=mk

if [ -n "$1" ]; then
    if [ $1 = 'release' ]; then
        cargo build --release
        mode=release
    fi
else
    cargo build
fi

rm -f ./$bin
cp target/$mode/$bin $bin

$priv chown root ./$bin
$priv chmod 4555 ./$bin
