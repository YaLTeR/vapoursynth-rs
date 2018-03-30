//! Things related to making VapourSynth plugins.
use failure::Error;
use std::ffi::CString;
use std::{mem, panic, process, ptr};
use std::ops::DerefMut;
use std::os::raw::c_void;
use vapoursynth_sys as ffi;

use api::API;
use core::CoreRef;
use frame::Frame;
use map::Map;
use video_info::VideoInfo;

mod frame_context;
pub use self::frame_context::FrameContext;

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

/// A filter interface.
pub trait Filter: Sized + Send + Sync {
    /// Returns the name of the filter.
    ///
    /// The characters allowed are letters, numbers, and the underscore. The first character must
    /// be a letter. In other words: `^[a-zA-Z][a-zA-Z0-9_]*$`.
    ///
    /// For example, `Invert`.
    fn name() -> &'static str;

    /// Returns the filter argument string.
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
    /// `Filter::args()`. Their presence and types are validated by VapourSynth so it's safe to
    /// `unwrap()`.
    ///
    /// In this function you should take all input nodes for your filter and store them somewhere
    /// so that you can request their frames in `get_frame_initial()`.
    fn create(api: API, core: CoreRef, args: &Map) -> Result<Self, Error>;

    /// Returns the parameters of this filter's output node.
    ///
    /// The returned vector should contain one entry for each node output index.
    fn video_info(&self, api: API, core: CoreRef) -> Vec<VideoInfo>;

    /// Requests the necessary frames from downstream nodes.
    ///
    /// This is always the first function to get called for a given frame `n`.
    ///
    /// In this function you should call `request_frame_filter()` on any input nodes that you need.
    ///
    /// Do not call `Node::get_frame()` from within this function.
    fn get_frame_initial(
        &self,
        api: API,
        core: CoreRef,
        context: FrameContext,
        n: usize,
    ) -> Result<(), Error>;

    /// Returns the requested frame.
    ///
    /// This is always the second function to get called for a given frame `n`.
    ///
    /// In this function you should call `get_frame_filter()` on the input nodes to retrieve the
    /// frames you requested in `get_frame_initial()`.
    ///
    /// Do not call `Node::get_frame()` from within this function.
    fn get_frame(
        &self,
        api: API,
        core: CoreRef,
        context: FrameContext,
        n: usize,
    ) -> Result<Frame, Error>;
}

/// Pushes the error backtrace into the given string.
fn push_backtrace(buf: &mut String, err: &Error) {
    for cause in err.causes().skip(1) {
        buf.push_str(&format!("Caused by: {}", cause));
    }

    buf.push_str(&format!("{}", err.backtrace()));
}

unsafe extern "system" fn init<F: Filter>(
    _in_: *mut ffi::VSMap,
    out: *mut ffi::VSMap,
    instance_data: *mut *mut c_void,
    node: *mut ffi::VSNode,
    core: *mut ffi::VSCore,
    _vsapi: *const ffi::VSAPI,
) {
    let closure = move || {
        let core = CoreRef::from_ptr(core);
        let filter = Box::from_raw(*(instance_data as *mut *mut F));

        let vi = filter
            .video_info(API::get_cached(), core)
            .into_iter()
            .map(VideoInfo::ffi_type)
            .collect::<Vec<_>>();
        API::get_cached().set_video_info(&vi, node);

        mem::forget(filter);
    };

    if panic::catch_unwind(closure).is_err() {
        let closure = move || {
            // We have to leak filter here because we can't guarantee that it's in a consistent
            // state after a panic.
            //
            // Just set the error message.
            let out = Map::from_mut_ptr(out);
            out.set_error(&format!("Panic during init() of {}", F::name()));
        };

        if panic::catch_unwind(closure).is_err() {
            process::abort();
        }
    }
}

unsafe extern "system" fn free<F: Filter>(
    instance_data: *mut c_void,
    core: *mut ffi::VSCore,
    _vsapi: *const ffi::VSAPI,
) {
    let closure = move || {
        let filter = Box::from_raw(instance_data as *mut F);
        drop(filter);
    };

    if panic::catch_unwind(closure).is_err() {
        process::abort();
    }
}

unsafe extern "system" fn get_frame<F: Filter>(
    n: i32,
    activation_reason: i32,
    instance_data: *mut *mut c_void,
    _frame_data: *mut *mut c_void,
    frame_ctx: *mut ffi::VSFrameContext,
    core: *mut ffi::VSCore,
    _vsapi: *const ffi::VSAPI,
) -> *const ffi::VSFrameRef {
    let closure = move || {
        let api = API::get_cached();
        let core = CoreRef::from_ptr(core);
        let context = FrameContext::from_ptr(frame_ctx);

        let filter = Box::from_raw(*(instance_data as *mut *mut F));

        debug_assert!(n >= 0);
        let n = n as usize;

        let rv = match activation_reason {
            x if x == ffi::VSActivationReason::arInitial as _ => {
                if let Err(err) = filter.get_frame_initial(api, core, context, n) {
                    let mut buf = String::new();

                    buf.push_str(&format!(
                        "Error in Filter::get_frame_initial(): {}",
                        err.cause()
                    ));

                    push_backtrace(&mut buf, &err);

                    let buf = CString::new(buf.replace('\0', "\\0")).unwrap();
                    api.set_filter_error(buf.as_ptr(), frame_ctx);
                }

                ptr::null()
            }
            x if x == ffi::VSActivationReason::arAllFramesReady as _ => {
                match filter.get_frame(api, core, context, n) {
                    Ok(frame) => {
                        let ptr = frame.ptr();
                        // The ownership is transferred to the caller.
                        mem::forget(frame);
                        ptr
                    }
                    Err(err) => {
                        let mut buf = String::new();

                        buf.push_str(&format!("Error in Filter::get_frame(): {}", err.cause()));

                        push_backtrace(&mut buf, &err);

                        let buf = CString::new(buf.replace('\0', "\\0")).unwrap();
                        api.set_filter_error(buf.as_ptr(), frame_ctx);

                        ptr::null()
                    }
                }
            }
            _ => ptr::null(),
        };

        mem::forget(filter);

        rv
    };

    match panic::catch_unwind(closure) {
        Ok(frame) => frame,
        Err(_) => process::abort(),
    }
}

unsafe extern "system" fn create<F: Filter>(
    in_: *const ffi::VSMap,
    out: *mut ffi::VSMap,
    user_data: *mut c_void,
    core: *mut ffi::VSCore,
    api: *const ffi::VSAPI,
) {
    let name = Box::from_raw(user_data as *mut CString);
    let name_cstr = name.as_ref();

    API::set(api);

    let closure = move || {
        let args = Map::from_ptr(in_);
        let out = Map::from_mut_ptr(out);
        let core = CoreRef::from_ptr(core);

        let filter = match F::create(API::get_cached(), core, args) {
            Ok(filter) => Box::new(filter),
            Err(err) => {
                let mut buf = String::new();

                buf.push_str(&format!(
                    "Error in Filter::create() of {}: {}",
                    name_cstr.to_str().unwrap(),
                    err.cause()
                ));

                push_backtrace(&mut buf, &err);

                out.set_error(&buf.replace('\0', "\\0")).unwrap();
                return;
            }
        };

        API::get_cached().create_filter(
            in_,
            out.deref_mut(),
            name_cstr.as_ptr(),
            init::<F>,
            get_frame::<F>,
            Some(free::<F>),
            ffi::VSFilterMode::fmParallel,
            ffi::VSNodeFlags(0),
            Box::into_raw(filter) as *mut _,
            core.ptr(),
        );
    };

    if panic::catch_unwind(closure).is_err() {
        let closure = move || {
            let out = Map::from_mut_ptr(out);
            out.set_error(&format!(
                "Panic during Filter::create() of {}",
                name_cstr.to_str().unwrap()
            ));
        };

        if panic::catch_unwind(closure).is_err() {
            process::abort();
        }
    }
}

/// Registers the plugin.
///
/// This function is for internal use only.
///
/// # Safety
/// The caller must ensure the pointers are valid.
#[inline]
pub unsafe fn call_config_func(
    config_func: *const c_void,
    plugin: *mut c_void,
    metadata: Metadata,
) {
    let config_func = *(&config_func as *const _ as *const ffi::VSConfigPlugin);

    let identifier_cstring = CString::new(metadata.identifier)
        .expect("Couldn't convert the plugin identifier to a CString");
    let namespace_cstring = CString::new(metadata.namespace)
        .expect("Couldn't convert the plugin namespace to a CString");
    let name_cstring =
        CString::new(metadata.name).expect("Couldn't convert the plugin name to a CString");

    config_func(
        identifier_cstring.as_ptr(),
        namespace_cstring.as_ptr(),
        name_cstring.as_ptr(),
        ffi::VAPOURSYNTH_API_VERSION,
        if metadata.read_only { 1 } else { 0 },
        plugin as *mut ffi::VSPlugin,
    );
}

/// Registers the filter `F`.
///
/// This function is for internal use only.
///
/// # Safety
/// The caller must ensure the pointers are valid.
#[inline]
pub unsafe fn call_register_func<F: Filter>(register_func: *const c_void, plugin: *mut c_void) {
    let register_func = *(&register_func as *const _ as *const ffi::VSRegisterFunction);

    let name_cstring =
        CString::new(F::name()).expect("Couldn't convert the filter name to a CString");
    let args_cstring =
        CString::new(F::args()).expect("Couldn't convert the filter args to a CString");

    let name = Box::new(name_cstring);

    register_func(
        name.as_ptr(),
        args_cstring.as_ptr(),
        create::<F>,
        Box::into_raw(name) as *mut c_void,
        plugin as *mut ffi::VSPlugin,
    );
}

/// Exports a VapourSynth plugin from this library.
///
/// This macro should be used only once at the top level of the library. The library should have a
/// `cdylib` crate type.
///
/// The first parameter is a `Metadata` expression containing your plugin's metadata.
///
/// Following it is a list of types implementing `Filter`, those are the filters the plugin will
/// export.
///
/// # Example
/// ```no_compile
/// export_vapoursynth_plugin! {
///     Metadata {
///         identifier: "com.example.invert",
///         namespace: "invert",
///         name: "Invert Example Plugin",
///         read_only: true,
///     },
///     [SampleFilter]
/// }
/// ```
#[macro_export]
macro_rules! export_vapoursynth_plugin {
    ($metadata:expr, [$($filter:ty),*]) => (
        use ::std::os::raw::c_void;

        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "system" fn VapourSynthPluginInit(
            config_func: *const c_void,
            register_func: *const c_void,
            plugin: *mut c_void,
        ) {
            use ::std::{panic, process};
            use $crate::plugins::{call_config_func, call_register_func};

            let closure = move || {
                call_config_func(config_func, plugin, $metadata);

                $(call_register_func::<$filter>(register_func, plugin);)*
            };

            if panic::catch_unwind(closure).is_err() {
                process::abort();
            }
        }
    )
}
