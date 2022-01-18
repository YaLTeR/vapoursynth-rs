#!/bin/sh
set -ex

# Install nasm
sudo apt-get install nasm

# Install a 32-bit environment if a 32-bit arch is used
if [ "$1" = "i686" ]; then
    sudo dpkg --add-architecture i386
    sudo apt-get update
    sudo apt-get install gcc-multilib g++-multilib libpython3.8-dev:i386
fi

# Install Cython
sudo pip3 install cython

# Change the configure arguments according to the architecture
if [ "$1" = "i686" ]; then
    CONFIGURE_ARGS="--build=i686-linux-gnu \
                    CFLAGS=-m32 CXXFLAGS=-m32 LDFLAGS=-m32"
else
    CONFIGURE_ARGS=""
fi

# Install zimg
git clone --depth 1 --branch release-3.0.3 https://github.com/sekrit-twc/zimg.git
cd zimg
./autogen.sh
./configure $CONFIGURE_ARGS
sudo make install

cd ..

# Install VapourSynth
git clone --depth 1 --branch R53 https://github.com/vapoursynth/vapoursynth.git vs-dir
cd vs-dir
./autogen.sh
./configure $CONFIGURE_ARGS
sudo make install

cd ..

# Set VapourSynth environment
sudo ldconfig /usr/local/lib
PYTHON3_LOCAL_LIB_PATH=$(echo /usr/local/lib/python3.*)
SITE=$PYTHON3_LOCAL_LIB_PATH/site-packages/vapoursynth.so
DIST=$PYTHON3_LOCAL_LIB_PATH/dist-packages/vapoursynth.so
sudo ln -s "$SITE" "$DIST"
