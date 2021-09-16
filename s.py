#!/usr/bin/env python

"""
Checks for supported features by compiling against their system requirements.
"""

import os
import sys
import hashlib


# Temporary build files
TMP = 'target/supported'

# Feature requirements
FEATURES = {
    '': {
        'headers': ['pwd'],
        'dylibs': []
    },
    'shadow': {
        'headers': ['shadow'],
        'dylibs': []
    },
    'pam': {
        'headers': ['security/pam_appl'],
        'dylibs': ['pam']
    }
}


def test_cc(name: str, src: str, links) -> str:
    path = os.path.join(TMP, f'{name.replace("/", "__")}.c')
    out = os.path.join(
        TMP, f'__build_{hashlib.md5(bytes(src, "utf-8")).hexdigest()}.out')

    # Don't recompile if we already have an output
    if os.path.exists(out):
        return out

    with open(path, 'w') as f:
        f.write(src)

    cmd = f'cc {path} {" ".join([f"-l{lib}" for lib in links])} -o {out} 2> /dev/null'

    # 0 expected
    if not os.system(cmd):
        return out

    return None


def maybe_feature(feat: str) -> bool:
    if feat in FEATURES:
        links = []
        headers = []
        defs = []

        # Get required libraries to link to
        if 'dylibs' in FEATURES[feat]:
            links.extend(FEATURES[feat]['dylibs'])

        # Get required header files to include
        if 'headers' in FEATURES[feat]:
            headers.extend(FEATURES[feat]['headers'])

        # Get required definitions
        if 'defs' in FEATURES[feat]:
            defs.extend(FEATURES[feat]['defs'])

        # `f-string expression part cannot include a backslash`
        ln = '\n'

        # Generate source file
        src = f"""
{ ln.join(f'#include <{h}.h>' for h in headers) }

int main(int argc, char** argv) {'{'}
    { ln.join([(f'#ifndef {d[1::]}' if d.startswith('!') else f'#ifdef {d}') for d in defs]) }
    return 0;
    { ln.join(['#endif' for _ in defs]) }
    return -1;
{'}'}
"""

        x = test_cc(feat, src, links)

        # 0 expected
        if not x or os.system(f'sh -c {x} 2> /dev/null'):
            return False

    return True


def main():
    if not os.path.exists(TMP):
        os.makedirs(TMP)

    flist = []

    for f in FEATURES:
        if maybe_feature(f):
            flist.append(f)
        else:
            if f == '':
                sys.exit(-1)

    print(','.join(flist))


if __name__ == '__main__':
    main()
