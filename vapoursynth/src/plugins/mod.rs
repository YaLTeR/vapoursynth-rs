//! Things related to making VapourSynth plugins.
use failure::Error;

use api::API;
use core::CoreRef;
use frame::FrameRef;
use map::Map;
use video_info::VideoInfo;

mod frame_context;
pub use self::frame_context::FrameContext;

pub mod ffi;

/// Plugin metadata.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Metadata {
    /// A "reverse" URL, unique among all plugins.
    ///
    /// For example, `com.example.invert`.
    pub identifier: &'static str,

    /// Namespace where the plugin's filters will go, unique among all plugins.
    ///
    /// Only lowercase letters and the underscore should be used, and it shouldn't be too long.
    /// Additionally, words that are special to Python, e.g. `del`, should be avoided.
    ///
    /// For example, `invert`.
    pub namespace: &'static str,

    /// Plugin name in readable form.
    ///
    /// For example, `Invert Example Plugin`.
    pub name: &'static str,

    /// Whether new filters can be registered at runtime.
    ///
    /// This should generally be set to `false`. It's used for the built-in AviSynth compat plugin.
    pub read_only: bool,
}

/// A filter function interface.
pub trait FilterFunction {
    /// Returns the name of the function.
    ///
    /// The characters allowed are letters, numbers, and the underscore. The first character must
    /// be a letter. In other words: `^[a-zA-Z][a-zA-Z0-9_]*$`.
    ///
    /// For example, `Invert`.
    fn name() -> &'static str;

    /// Returns the argument string.
    ///
    /// Arguments are separated by a semicolon. Each argument is made of several fields separated
    /// by a colon. Don’t insert additional whitespace characters, or VapourSynth will die.
    ///
    /// Fields:
    /// - The argument name. The same characters are allowed as for the filter's name. Argument
    ///   names should be all lowercase and use only letters and the underscore.
    ///
    /// - The type. One of `int`, `float`, `data`, `clip`, `frame`, `func`. They correspond to the
    ///   `Map::get_*()` functions (`clip` is `get_node()`). It's possible to declare an array by
    ///   appending `[]` to the type.
    ///
    /// - `opt` if the parameter is optional.
    ///
    /// - `empty` if the array is allowed to be empty.
    ///
    /// The following example declares the arguments "blah", "moo", and "asdf":
    /// `blah:clip;moo:int[]:opt;asdf:float:opt;`
    // TODO: automate this. Filters should have their `create()` function accept the arguments
    // directly, and there should be a custom derive or something that generates the argument
    // string.
    fn args() -> &'static str;

    /// Creates a new instance of the filter and returns it.
    ///
    /// `args` contains the filter arguments, as specified by the argument string from
    /// `FilterFunction::args()`. Their presence and types are validated by VapourSynth so it's
    /// safe to `unwrap()`.
    ///
    /// In this function you should take all input nodes for your filter and store them somewhere
    /// so that you can request their frames in `get_frame_initial()`.
    fn create<'core>(
        api: API,
        core: CoreRef<'core>,
        args: &Map<'core>,
    ) -> Result<Box<Filter<'core> + 'core>, Error>;
}

/// A filter interface.
pub trait Filter<'core>: Send + Sync {
    /// Returns the parameters of this filter's output node.
    ///
    /// The returned vector should contain one entry for each node output index.
    fn video_info(&self, api: API, core: CoreRef<'core>) -> Vec<VideoInfo<'core>>;

    /// Requests the necessary frames from downstream nodes.
    ///
    /// This is always the first function to get called for a given frame `n`.
    ///
    /// In this function you should call `request_frame_filter()` on any input nodes that you need
    /// and return `None`. If you do not need any input frames, you should generate the output
    /// frame and return it here.
    ///
    /// Do not call `Node::get_frame()` from within this function.
    fn get_frame_initial(
        &self,
        api: API,
        core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<Option<FrameRef<'core>>, Error>;

    /// Returns the requested frame.
    ///
    /// This is always the second function to get called for a given frame `n`. If the frame was
    /// retrned from `get_frame_initial()`, this function is not called.
    ///
    /// In this function you should call `get_frame_filter()` on the input nodes to retrieve the
    /// frames you requested in `get_frame_initial()`.
    ///
    /// Do not call `Node::get_frame()` from within this function.
    fn get_frame(
        &self,
        api: API,
        core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<FrameRef<'core>, Error>;
}
