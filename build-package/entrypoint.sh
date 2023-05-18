#!/bin/bash

sudo chmod -R 777 /github/home $GITHUB_WORKSPACE
sudo chmod 666 $GITHUB_OUTPUT

python -u /buildpkg.py "$@"

ls -R ~
if [ -d "~/.cache/paru" ]; then
	ls -R ~/.cache/paru/clone
	sudo cp ~/.cache/paru/clone/*/*.pkg.tar.zst $GITHUB_WORKSPACE/
fi
