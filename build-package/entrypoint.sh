#!/bin/bash

sudo chmod -R 777 /github/home
sudo chmod 666 $GITHUB_OUTPUT

python -u /buildpkg.py "$@"
