#!/usr/bin/env sh

# Build and run `mk` with expected permissions.
# (Disclaimer: I don't know how to shell script)

set -e

MODE=debug
BIN=mk
PRIV=doas
DIR=$(dirname $(realpath $0))/../

cd $DIR

# Use `mk` if it is in the path ( ͡° ͜ʖ ͡°)-b
if [ $(which mk 2>/dev/null) ]; then
    PRIV=mk
elif [ $(which sudo 2>/dev/null) ]; then
    PRIV=sudo
fi

# Build `mk`
if [ -n "$1" ]; then
    if [ $1 = 'release' ]; then
        cargo build --release
        MODE=release
    fi
else
    cargo build
fi

# Copy `mk`
cp -f target/$MODE/$BIN $BIN

# Set permissions
$PRIV chown root ./$BIN
$PRIV chmod 4555 ./$BIN

cd $PWD
