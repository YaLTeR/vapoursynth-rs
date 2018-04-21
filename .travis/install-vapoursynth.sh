#!/bin/sh
set -ex

if [ "$TRAVIS_OS_NAME" = "osx" ]; then
	brew install vapoursynth
else
	exit 1
fi
