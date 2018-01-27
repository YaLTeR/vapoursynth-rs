# vapoursynth-rs

A (very WIP) safe wrapper for [VapourSynth](https://github.com/vapoursynth/vapoursynth), written in Rust.

The primary goal is safety (that is, safe Rust code should not trigger undefined behavior), and secondary goals include performance and ease of use.

## Supported Versions

Right now the x86_64 and i686 Unix and x86_64 Windows targets are supported.

All VapourSynth and VSScript API versions starting with 3.0 are supported. By default the crates use the 3.0 feature set. To enable higher API version support, enable one of the following Cargo features:

* `vapoursynth-api-31` for VapourSynth API 3.1
* `vapoursynth-api-32` for VapourSynth API 3.2
* `vapoursynth-api-33` for VapourSynth API 3.3
* `vapoursynth-api-34` for VapourSynth API 3.4
* `vapoursynth-api-35` for VapourSynth API 3.5

* `vsscript-api-31` for VSScript API 3.5

To enable linking to VapourSynth or VSScript functions (currently required to do anything useful), enable the following Cargo features:

* `vapoursynth-functions` for VapourSynth functions (`getVapourSynthAPI()`)
* `vsscript-functions` for VSScript functions (`vsscript_*()`)

## Building

Make sure you have the corresponding libraries available if you enable the linking features. You can use the `VAPOURSYNTH_LIB_DIR` environment variable to specify a custom directory with the library files.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
