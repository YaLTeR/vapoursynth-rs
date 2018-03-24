# vapoursynth-rs

[![crates.io](https://img.shields.io/crates/v/vapoursynth.svg)](https://crates.io/crates/vapoursynth)
[![Documentation](https://docs.rs/vapoursynth/badge.svg)](https://docs.rs/vapoursynth)
[![Travis Build Status](https://api.travis-ci.org/YaLTeR/vapoursynth-rs.svg?branch=master)](https://travis-ci.org/YaLTeR/vapoursynth-rs)
[![AppVeyor Build Status](https://ci.appveyor.com/api/projects/status/kwyhlamoqje8tsqc?svg=true)](https://ci.appveyor.com/project/YaLTeR/vapoursynth-rs)

[ChangeLog](https://github.com/YaLTeR/vapoursynth-rs/blob/master/vapoursynth/CHANGELOG.md)
[Documentation for the master branch with all features enabled](https://yalter.github.io/vapoursynth-rs)

A safe wrapper for [VapourSynth](https://github.com/vapoursynth/vapoursynth), written in Rust.

The primary goal is safety (that is, safe Rust code should not trigger undefined behavior), and secondary goals include performance and ease of use.

## Functionality

Most of the VapourSynth API is covered. It's possible to evaluate `.vpy` scripts, access their properties and output, retrieve frames. A notable exception is API for creating VapourSynth filters, which will come out next.

For an example usage see [examples/vspipe.rs](https://github.com/YaLTeR/vapoursynth-rs/blob/master/vapoursynth/examples/vspipe.rs), a complete reimplementation of VapourSynth's [vspipe](https://github.com/vapoursynth/vapoursynth/blob/master/src/vspipe/vspipe.cpp) in safe Rust utilizing this crate.

## vapoursynth-sys

[![crates.io](https://img.shields.io/crates/v/vapoursynth-sys.svg)](https://crates.io/crates/vapoursynth-sys)
[![Documentation](https://docs.rs/vapoursynth-sys/badge.svg)](https://docs.rs/vapoursynth-sys)

[ChangeLog](https://github.com/YaLTeR/vapoursynth-rs/blob/master/vapoursynth-sys/CHANGELOG.md)

Raw bindings to [VapourSynth](https://github.com/vapoursynth/vapoursynth).

## Supported Versions

All VapourSynth and VSScript API versions starting with 3.0 are supported. By default the crates use the 3.0 feature set. To enable higher API version support, enable one of the following Cargo features:

* `vapoursynth-api-31` for VapourSynth API 3.1
* `vapoursynth-api-32` for VapourSynth API 3.2
* `vapoursynth-api-33` for VapourSynth API 3.3
* `vapoursynth-api-34` for VapourSynth API 3.4
* `vapoursynth-api-35` for VapourSynth API 3.5
* `vsscript-api-31` for VSScript API 3.1
* `vsscript-api-32` for VSScript API 3.2

To enable linking to VapourSynth or VSScript functions (currently required to do anything useful), enable the following Cargo features:

* `vapoursynth-functions` for VapourSynth functions (`getVapourSynthAPI()`)
* `vsscript-functions` for VSScript functions (`vsscript_*()`)

## Building

Make sure you have the corresponding libraries available if you enable the linking features. You can use the `VAPOURSYNTH_LIB_DIR` environment variable to specify a custom directory with the library files.

On Windows the easiest way is to use the VapourSynth installer (make sure the VapourSynth SDK is checked). The crate should pick up the library directory automatically. If it doesn't or if you're cross-compiling, set `VAPOURSYNTH_LIB_DIR` to `<path to the VapourSynth installation>\sdk\lib64` or `<...>\lib32`, depending on the target bitness.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
