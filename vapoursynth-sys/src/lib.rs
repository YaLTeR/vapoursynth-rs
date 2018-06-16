//! Raw bindings to [VapourSynth](https://github.com/vapoursynth/vapoursynth).
#![doc(html_root_url = "https://docs.rs/vapoursynth-sys/0.2.2")]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

#[macro_use]
extern crate cfg_if;

mod bindings;
pub use bindings::*;

macro_rules! api_version {
    ($major:expr, $minor:expr) => {
        ($major << 16) | $minor
    };
}

cfg_if! {
    if #[cfg(feature="vapoursynth-api-35")] {
        pub const VAPOURSYNTH_API_VERSION: i32 = api_version!(3, 5);
    } else if #[cfg(feature="vapoursynth-api-34")] {
        pub const VAPOURSYNTH_API_VERSION: i32 = api_version!(3, 4);
    } else if #[cfg(feature="vapoursynth-api-33")] {
        pub const VAPOURSYNTH_API_VERSION: i32 = api_version!(3, 3);
    } else if #[cfg(feature="vapoursynth-api-32")] {
        pub const VAPOURSYNTH_API_VERSION: i32 = api_version!(3, 2);
    } else if #[cfg(feature="vapoursynth-api-31")] {
        pub const VAPOURSYNTH_API_VERSION: i32 = api_version!(3, 1);
    } else {
        pub const VAPOURSYNTH_API_VERSION: i32 = api_version!(3, 0);
    }
}

cfg_if! {
    if #[cfg(feature="vsscript-api-32")] {
        pub const VSSCRIPT_API_VERSION: i32 = api_version!(3, 2);
    } else if #[cfg(feature="vsscript-api-31")] {
        pub const VSSCRIPT_API_VERSION: i32 = api_version!(3, 1);
    } else {
        pub const VSSCRIPT_API_VERSION: i32 = api_version!(3, 0);
    }
}
