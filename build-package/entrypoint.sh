#!/bin/bash

sudo chmod -R 777 /github/home $GITHUB_WORKSPACE
sudo chmod 666 $GITHUB_OUTPUT

python -u /buildpkg.py "$@"

ls -aR ~
if [[ -d "~/.cache/paru" ]]; then
	echo "In the if"
	ls -R ~/.cache/paru/clone
	sudo cp ~/.cache/paru/clone/*/*.pkg.tar.zst $GITHUB_WORKSPACE/
fi
