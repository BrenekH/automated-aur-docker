#!/bin/bash

pacman -U --noconfirm $GITHUB_WORKSPACE/*.pkg.tar.zst

python -u /testpkg.py "$@"
