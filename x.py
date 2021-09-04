#!/usr/bin/env python

"""
Helper script for building, installing, and testing `mk`.
"""

import os
import argparse

# Backwards compatibility
from typing import List

# Constants
BIN = 'mk'
DIR = os.path.dirname(os.path.abspath(__file__))
OLD = os.getcwd()

# Privilege giving binary
SUID = ''
for i in ['/usr/bin/sudo', '/usr/bin/doas']:
    if os.path.exists(i):
        SUID = i
        break

FEATURES = {
    # --- These are required ---
    '': {
        'headers': [],
        'dylibs': ['crypt']
    },
    # --------------------------
    'sdw': {
        'headers': ['shadow'],
        'dylibs': []
    },
    'pam': {
        'headers': ['security/pam_appl'],
        'dylibs': ['pam']
    }
}

# Rudimentary methods to find headers and libraries

def find_header(header: str) -> bool:
    path = os.path.join('/usr/include/', f'{header}.h')
    return os.path.exists(path)

def find_dylib(lib: str) -> bool:
    path = os.path.join('/usr/lib/', f'lib{lib}.so')
    return os.path.exists(path)

# Build and install related functions

def maybe_feature(feature: str) -> bool:
    """Validate if all files required for a feature are available."""

    if feature in FEATURES:
        # Check if all required header files are available
        for h in FEATURES[feature]['headers']:
            if not find_header(h):
                return False

        # Check if all required libraries are available
        for l in FEATURES[feature]['dylibs']:
            if not find_dylib(l):
                return False

    return True

def setup_permissions(path: str):
    """
    Change ownership of a file to root, and set its permissions to:
    ```
    .rws--x--x
    ```
    """
    os.system(f'{SUID} chown root {path}')
    os.system(f'{SUID} chmod 4711 {path}')

def clean():
    """Clean the build environment using ``cargo``."""

    os.system('cargo clean')

def test(bin: str, features: List[str]):
    """Test all crates in the workspace using ``cargo``."""

    # Collect features
    features_list = ''
    for f in features:
        features_list += f' {f}'

    os.system(f'cargo test --workspace --features "{features_list}"')

def build(bin: str, rel: bool, features: List[str]):
    """Build the binary using ``cargo``."""

    print(f"Building ``{bin}`` with features: {features} ...")

    # Collect features
    features_list = ''
    for f in features:
        features_list += f' {f}'

    # Build
    os.system(f'cargo build {"--release" if rel else ""} --features "{features_list}"')

    # Copy file
    os.system(f'{SUID} cp -f target/{"release" if rel else "debug"}/{BIN} {DIR}/{BIN}')
    setup_permissions(f'{DIR}/{BIN}')

def install(bin: str, features: List[str]):
    """Build and install the binary using ``cargo``."""

    # Build binary
    build(bin, True, features)

    # Copy file
    os.system(f'{SUID} cp -f target/release/{BIN} /usr/bin/{BIN}')
    setup_permissions(f'/usr/bin/{BIN}')

# Command line options

def create_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Build and install options for ``mk``.")
    parser.add_argument('--bin', nargs='?', type=str, help="Name of the binary target to build.")
    parser.add_argument('--clean', action='store_true', help="Clean the build environment using ``cargo``.")
    parser.add_argument('--test', action='store_true', help="Test all crates in the workspace using ``cargo``.")
    parser.add_argument('--build', nargs='?', const='debug', type=str, help="Mode to build the binary in.")
    parser.add_argument('--install', action='store_true', help="Install the binary to ``/usr/bin``.")
    return parser

def main():
    # We'll operate in the project root for simplicity
    os.chdir(DIR)

    # Collect available features
    features = []
    for f in FEATURES:
        if maybe_feature(f):
            features.append(f)

    # Current binary target
    current_bin = BIN

    # Parse arguments
    args = create_parser().parse_args()

    if args.bin:
        current_bin = args.bin

    if args.clean:
        clean()

    if args.test:
        test(current_bin, features)

    if args.build:
        build(current_bin, args.build == 'release', features)

    if args.install:
        install(current_bin, features)

    # Go back to the original working directory
    os.chdir(OLD)

if __name__ == '__main__':
    main()
