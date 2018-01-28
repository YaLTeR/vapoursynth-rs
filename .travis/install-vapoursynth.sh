#!/bin/sh
set -e

sudo apt-get install -qq software-properties-common
sudo add-apt-repository -y ppa:djcj/vapoursynth
sudo apt-get update -qq
sudo apt-get install -qq vapoursynth
