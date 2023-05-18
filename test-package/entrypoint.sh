#!/bin/bash

pacman -Syu

# We "install" the built packages twice, once without dependency checks and once with, so
# that if the shared package cache contains packages that depend on each other, Pacman will
# still install the packages in the cache, and the package dependencies (using the second)
# install command.
pacman -Udd --noconfirm $GITHUB_WORKSPACE/*.pkg.tar.zst
pacman -U --noconfirm $GITHUB_WORKSPACE/*.pkg.tar.zst

python -u /testpkg.py "$@"
