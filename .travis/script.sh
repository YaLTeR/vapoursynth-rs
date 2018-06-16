#!/bin/sh
set -ex

# Test with all possible combinations of all features.
cd vapoursynth; python3 ../.travis/run-tests.py
cd ..

# Run sample plugin tests.
cd sample-plugin
cargo build --verbose
cargo run --verbose --bin test \
    --features "cfg-if vapoursynth/vapoursynth-functions vapoursynth/vsscript-functions"
cd ..

# Doc with all features.
cargo doc --verbose --all-features
cp .travis/index.html target/doc/

# Remove the lock file that gets left over and screws over Travis deployment.
rm target/doc/.lock
