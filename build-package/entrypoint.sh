#!/bin/bash

sudo chmod -R 777 /github/home

python -u /buildpkg.py "$@"
