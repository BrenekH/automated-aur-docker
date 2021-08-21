#!/bin/bash

pacman -S --noconfirm $GITHUB_WORKSPACE/*.pkg.tar.zst

python -u /testpkg.py "$@"
