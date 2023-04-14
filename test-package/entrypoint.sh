#!/bin/bash

pacman -Syu
pacman -Udd --noconfirm $GITHUB_WORKSPACE/*.pkg.tar.zst

python -u /testpkg.py "$@"
