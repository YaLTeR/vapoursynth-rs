#!/bin/sh
set -ex

if [ "$TRAVIS_OS_NAME" = "osx" ]; then
	brew install vapoursynth
else
	sudo add-apt-repository -y ppa:djcj/vapoursynth
	sudo apt-get update -qq

	if [ "$TARGET_ARCH" = "i686" ]; then
		sudo apt-get install -qq vapoursynth:i386 gcc-multilib
		sudo ln -s libvapoursynth-script.so.0 /usr/lib/i386-linux-gnu/libvapoursynth-script.so
	else
		sudo apt-get install -qq vapoursynth
		sudo ln -s libvapoursynth-script.so.0 /usr/lib/x86_64-linux-gnu/libvapoursynth-script.so
	fi
fi
