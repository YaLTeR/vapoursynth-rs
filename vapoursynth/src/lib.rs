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
pub use api::{MessageType, API};

pub mod core;
pub use core::CoreRef;

pub mod format;
pub use format::Format;

pub mod frame;
pub use frame::Frame;

pub mod function;
pub use function::Function;

pub mod map;
pub use map::*;

pub mod node;
pub use node::Node;

pub mod video_info;
pub use video_info::Property;

mod tests;
