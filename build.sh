#!/usr/bin/env sh

# Build and run `mk` with expected permissions.

set -e

mode=debug

if [ -n "$1" ]; then
    if [ $1 = 'release' ]; then
        cargo build --release
        mode=release
    fi
else
    cargo build
fi

rm -f ./mk
cp target/$mode/mk mk

sudo chown root ./mk
sudo chmod 4555 ./mk
