#!/usr/bin/env sh

set -e

# Install `mk`

OUT=/usr/bin
BIN=mk
PRIV=doas
MODE=release
DIR=$(dirname $(realpath $0))/../

cd $DIR

# Use `mk` if it is in the path ( ͡° ͜ʖ ͡°)-b
if [ $(which mk 2>/dev/null) ]; then
    PRIV=mk
elif [ $(which sudo 2>/dev/null) ]; then
    PRIV=sudo
fi

cargo build --release

# Copy `mk`
$PRIV cp -f target/$MODE/$BIN $OUT/$BIN

# Set permissions
$PRIV chown root $OUT/$BIN
$PRIV chmod 4555 $OUT/$BIN

cd $PWD
