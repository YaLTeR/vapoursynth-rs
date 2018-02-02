#!/bin/bash
set -ex

if [[ $TRAVIS_OS_NAME == 'osx' ]]; then
	brew install vapoursynth
else
	sudo add-apt-repository -y ppa:djcj/vapoursynth
	sudo apt-get update -qq
	sudo apt-get install -qq vapoursynth
	sudo ln -s /usr/lib/x86_64-linux-gnu/libvapoursynth-script.so{.0,}
fi
