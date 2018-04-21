#!/bin/sh
set -ex

curl -sSf https://build.travis-ci.org/files/rustup-init.sh \
| sh -s -- --default-toolchain=$TRAVIS_RUST_VERSION -y

. ~/.cargo/env

if [ "$TARGET_ARCH" = "i686" ]; then
    rustup target add i686-unknown-linux-gnu
    export CARGO_BUILD_TARGET=i686-unknown-linux-gnu
fi

cd vapoursynth-rs
exec sh .travis/script.sh
