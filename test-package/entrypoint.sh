#!/bin/bash
set -euo pipefail

pacman -Syu

# List out copied packages
printf '%s\n' $GITHUB_WORKSPACE/*.pkg.tar*

# We "install" the built packages twice, once without dependency checks and once with, so
# that if the shared package cache contains packages that depend on each other, Pacman will
# still install the packages in the cache, and the package dependencies (using the second)
# install command.
echo "First install (no dependency checks)"
pacman -Udd --noconfirm $GITHUB_WORKSPACE/*.pkg.tar*
echo "Second install (install missing dependencies)"
pacman -U --noconfirm $GITHUB_WORKSPACE/*.pkg.tar*

/test-package "$@"
