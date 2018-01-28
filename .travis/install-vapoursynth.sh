#!/bin/sh
set -ex

sudo add-apt-repository -y ppa:djcj/vapoursynth
sudo apt-get update -qq
sudo apt-get install -qq vapoursynth
sudo ln -s /usr/lib/x86_64-linux-gnu/libvapoursynth-script.so{.0,}
