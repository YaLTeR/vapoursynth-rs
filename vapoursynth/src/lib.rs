//!  A safe wrapper for [VapourSynth](https://github.com/vapoursynth/vapoursynth), written in Rust.
//!
//! The primary goal is safety (that is, safe Rust code should not trigger undefined behavior), and
//! secondary goals include performance and ease of use.
//!
//! ## Functionality
//!
//! Most of the VapourSynth API is covered. It's possible to evaluate `.vpy` scripts, access their
//! properties and output, retrieve frames. A notable exception is API for creating VapourSynth
//! filters, which will come out next.
//!
//! For an example usage see
//! [examples/vspipe.rs](https://github.com/YaLTeR/vapoursynth-rs/blob/master/vapoursynth/examples/vspipe.rs),
//! a complete reimplementation of VapourSynth's
//! [vspipe](https://github.com/vapoursynth/vapoursynth/blob/master/src/vspipe/vspipe.cpp) in safe
//! Rust utilizing this crate.
//!
//! ## Short example
//!
//! ```no_run
//! # extern crate failure;
//! # extern crate vapoursynth;
//! # use failure::Error;
//! # #[cfg(all(feature = "vsscript-functions",
//! #           feature = "gte-vsscript-api-31",
//! #           any(feature = "vapoursynth-functions", feature = "gte-vsscript-api-32")))]
//! # fn foo() -> Result<(), Error> {
//! use vapoursynth::prelude::*;
//!
//! let env = Environment::from_file("test.vpy", EvalFlags::SetWorkingDir)?;
//! let node = env.get_output(0)?.0; // Without `.0` for VSScript API 3.0
//! let frame = node.get_frame(0)?;
//!
//! println!("Resolution: {}×{}", frame.width(0), frame.height(0));
//! # Ok(())
//! # }
//! # fn main() {
//! # }
//! ```
#![doc(html_root_url = "https://docs.rs/vapoursynth/0.1.0")]

#[macro_use]
extern crate bitflags;
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[cfg(any(not(feature = "gte-vsscript-api-32"), test))]
#[macro_use]
extern crate lazy_static;
extern crate vapoursynth_sys;

#[cfg(feature = "vsscript-functions")]
pub mod vsscript;

pub mod api;
pub mod core;
pub mod format;
pub mod frame;
pub mod function;
pub mod map;
pub mod node;
pub mod video_info;

pub mod prelude {
    //! The VapourSynth prelude.
    //!
    //! Contains the types you most likely want to import anyway.
    pub use super::api::{MessageType, API};
    pub use super::format::{ColorFamily, PresetFormat, SampleType};
    pub use super::frame::Frame;
    pub use super::map::{Map, OwnedMap, ValueType};
    pub use super::node::{GetFrameError, Node};
    pub use super::video_info::Property;

    #[cfg(feature = "vsscript-functions")]
    pub use super::vsscript::{self, Environment, EvalFlags};
}

mod tests;
