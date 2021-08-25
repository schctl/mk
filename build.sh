#!/usr/bin/env sh

# Build and run `mk` with expected permissions.

set -e

cargo build
mv target/debug/mk mk
sudo chown root mk
sudo chmod 4555 mk
