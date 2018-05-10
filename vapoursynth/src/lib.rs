//!  A safe wrapper for [VapourSynth](https://github.com/vapoursynth/vapoursynth), written in Rust.
//!
//! The primary goal is safety (that is, safe Rust code should not trigger undefined behavior), and
//! secondary goals include performance and ease of use.
//!
//! ## Functionality
//!
//! Most of the VapourSynth API is covered. It's possible to evaluate `.vpy` scripts, access their
//! properties and output, retrieve frames; enumerate loaded plugins and invoke their functions as
//! well as create VapourSynth filters.
//!
//! For an example usage see
//! [examples/vspipe.rs](https://github.com/YaLTeR/vapoursynth-rs/blob/master/vapoursynth/examples/vspipe.rs),
//! a complete reimplementation of VapourSynth's
//! [vspipe](https://github.com/vapoursynth/vapoursynth/blob/master/src/vspipe/vspipe.cpp) in safe
//! Rust utilizing this crate.
//!
//! For a VapourSynth plugin example see
//! [sample-plugin](https://github.com/YaLTeR/vapoursynth-rs/blob/master/sample-plugin) which
//! implements some simple filters.
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
//!
//! ## Plugins
//!
//! To make a VapourSynth plugin, start by creating a new Rust library with
//! `crate-type = ["cdylib"]`. Then add filters by implementing the `plugins::Filter` trait. Bind
//! them to functions by implementing `plugins::FilterFunction`, which is much more easily done via
//! the `make_filter_function!` macro. Finally, put `export_vapoursynth_plugin!` at the top level
//! of `src/lib.rs` to export the functionality.
//!
//! **Important note:** due to what seems to be a
//! [bug](https://github.com/rust-lang/rust/issues/50176) in rustc, it's impossible to make plugins
//! on the `i686-pc-windows-gnu` target (all other variations of `x86_64` and `i686` do work).
//! Please use `i686-pc-windows-msvc` for an i686 Windows plugin.
//!
//! ## Short plugin example
//!
//! ```no_run
//! #[macro_use]
//! extern crate failure;
//! #[macro_use]
//! extern crate vapoursynth;
//!
//! use failure::Error;
//! use vapoursynth::prelude::*;
//! use vapoursynth::core::CoreRef;
//! use vapoursynth::plugins::{Filter, FilterArgument, FrameContext, Metadata};
//! use vapoursynth::video_info::VideoInfo;
//!
//! // A simple filter that passes the frames through unchanged.
//! struct Passthrough<'core> {
//!     source: Node<'core>,
//! }
//!
//! impl<'core> Filter<'core> for Passthrough<'core> {
//!     fn video_info(&self, _api: API, _core: CoreRef<'core>) -> Vec<VideoInfo<'core>> {
//!         vec![self.source.info()]
//!     }
//!
//!     fn get_frame_initial(
//!         &self,
//!         _api: API,
//!         _core: CoreRef<'core>,
//!         context: FrameContext,
//!         n: usize,
//!     ) -> Result<Option<FrameRef<'core>>, Error> {
//!         self.source.request_frame_filter(context, n);
//!         Ok(None)
//!     }
//!
//!     fn get_frame(
//!         &self,
//!         _api: API,
//!         _core: CoreRef<'core>,
//!         context: FrameContext,
//!         n: usize,
//!     ) -> Result<FrameRef<'core>, Error> {
//!         self.source
//!             .get_frame_filter(context, n)
//!             .ok_or(format_err!("Couldn't get the source frame"))
//!     }
//! }
//!
//! make_filter_function! {
//!     PassthroughFunction, "Passthrough"
//!
//!     fn create_passthrough<'core>(
//!         _api: API,
//!         _core: CoreRef<'core>,
//!         clip: Node<'core>,
//!     ) -> Result<Option<Box<Filter<'core> + 'core>>, Error> {
//!         Ok(Some(Box::new(Passthrough { source: clip })))
//!     }
//! }
//!
//! export_vapoursynth_plugin! {
//!     Metadata {
//!         identifier: "com.example.passthrough",
//!         namespace: "passthrough",
//!         name: "Example Plugin",
//!         read_only: true,
//!     },
//!     [PassthroughFunction::new()]
//! }
//! # fn main() {
//! # }
//! ```
//!
//! Check [sample-plugin](https://github.com/YaLTeR/vapoursynth-rs/blob/master/sample-plugin) for
//! an example plugin which exports some simple filters.
//!
//! ## Supported Versions
//!
//! All VapourSynth and VSScript API versions starting with 3.0 are supported. By default the
//! crates use the 3.0 feature set. To enable higher API version support, enable one of the
//! following Cargo features:
//!
//! * `vapoursynth-api-31` for VapourSynth API 3.1
//! * `vapoursynth-api-32` for VapourSynth API 3.2
//! * `vapoursynth-api-33` for VapourSynth API 3.3
//! * `vapoursynth-api-34` for VapourSynth API 3.4
//! * `vapoursynth-api-35` for VapourSynth API 3.5
//! * `vsscript-api-31` for VSScript API 3.1
//! * `vsscript-api-32` for VSScript API 3.2
//!
//! To enable linking to VapourSynth or VSScript functions, enable the following Cargo features:
//!
//! * `vapoursynth-functions` for VapourSynth functions (`getVapourSynthAPI()`)
//! * `vsscript-functions` for VSScript functions (`vsscript_*()`)
//!
//! ## Building
//!
//! Make sure you have the corresponding libraries available if you enable the linking features.
//! You can use the `VAPOURSYNTH_LIB_DIR` environment variable to specify a custom directory with
//! the library files.
//!
//! On Windows the easiest way is to use the VapourSynth installer (make sure the VapourSynth SDK
//! is checked). The crate should pick up the library directory automatically. If it doesn't or if
//! you're cross-compiling, set `VAPOURSYNTH_LIB_DIR` to
//! `<path to the VapourSynth installation>\sdk\lib64` or `<...>\lib32`, depending on the target
//! bitness.

#![doc(html_root_url = "https://docs.rs/vapoursynth/0.2.0")]
// Preventing all those warnings with #[cfg] directives would be really diffucult.
#![allow(unused, dead_code)]

#[macro_use]
extern crate bitflags;
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[cfg(feature = "f16-pixel-type")]
extern crate half;
#[cfg(any(not(feature = "gte-vsscript-api-32"), test))]
#[macro_use]
extern crate lazy_static;
extern crate vapoursynth_sys;

#[cfg(feature = "vsscript-functions")]
pub mod vsscript;

pub mod api;
pub mod component;
pub mod core;
pub mod format;
pub mod frame;
pub mod function;
pub mod map;
pub mod node;
pub mod plugin;
pub mod plugins;
pub mod video_info;

pub mod prelude {
    //! The VapourSynth prelude.
    //!
    //! Contains the types you most likely want to import anyway.
    pub use super::api::{MessageType, API};
    pub use super::component::Component;
    pub use super::format::{ColorFamily, PresetFormat, SampleType};
    pub use super::frame::{Frame, FrameRef, FrameRefMut};
    pub use super::map::{Map, OwnedMap, ValueType};
    pub use super::node::{GetFrameError, Node};
    pub use super::plugin::Plugin;
    pub use super::video_info::Property;

    #[cfg(feature = "vsscript-functions")]
    pub use super::vsscript::{self, Environment, EvalFlags};
}

mod tests;
