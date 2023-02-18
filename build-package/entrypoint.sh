#!/bin/bash

sudo chmod -R 777 /github/home
sudo chmod 666 $GITHUB_OUTPUT

python -u /buildpkg.py "$@"

cp ~/.cache/paru/clone/*/*.pkg.tar.zst $GITHUB_WORKSPACE/
