//! Internal stuff for plugin FFI handling.
use failure::Error;
use std::ffi::CString;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_void;
use std::ptr::{self, NonNull};
use std::{mem, panic, process};
use vapoursynth_sys as ffi;

use api::API;
use core::CoreRef;
use frame::FrameRef;
use map::{Map, MapRef, MapRefMut};
use plugins::{Filter, FilterFunction, FrameContext, Metadata};
use video_info::VideoInfo;

/// Container for the internal filter function data.
pub(crate) struct FilterFunctionData<F: FilterFunction> {
    pub filter_function: F,
    // Store the name since it's supposed to be the same between two invocations (register and
    // create_filter).
    pub name: CString,
}

/// Pushes the error backtrace into the given string.
fn push_backtrace(buf: &mut String, err: &Error) {
    for cause in err.causes().skip(1) {
        buf.push_str(&format!("Caused by: {}", cause));
    }

    buf.push_str(&format!("{}", err.backtrace()));
}

/// Sets the video info of the output node of this filter.
unsafe extern "system" fn init(
    _in_: *mut ffi::VSMap,
    out: *mut ffi::VSMap,
    instance_data: *mut *mut c_void,
    node: *mut ffi::VSNode,
    core: *mut ffi::VSCore,
    _vsapi: *const ffi::VSAPI,
) {
    let closure = move || {
        let core = CoreRef::from_ptr(core);
        // The actual lifetime isn't 'static, it's 'core, but we don't really have a way of
        // retrieving it.
        let filter = Box::from_raw(*(instance_data as *mut *mut Box<Filter<'static> + 'static>));

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
            let mut out = MapRefMut::from_ptr(out);
            out.set_error("Panic during Filter::video_info()");
        };

        if panic::catch_unwind(closure).is_err() {
            process::abort();
        }
    }
}

/// Drops the filter.
unsafe extern "system" fn free(
    instance_data: *mut c_void,
    core: *mut ffi::VSCore,
    _vsapi: *const ffi::VSAPI,
) {
    let closure = move || {
        // The actual lifetime isn't 'static, it's 'core, but we don't really have a way of
        // retrieving it.
        let filter = Box::from_raw(instance_data as *mut Box<Filter<'static> + 'static>);
        drop(filter);
    };

    if panic::catch_unwind(closure).is_err() {
        process::abort();
    }
}

/// Calls `Filter::get_frame_initial()` and `Filter::get_frame()`.
unsafe extern "system" fn get_frame(
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

        // The actual lifetime isn't 'static, it's 'core, but we don't really have a way of
        // retrieving it.
        let filter = Box::from_raw(*(instance_data as *mut *mut Box<Filter<'static> + 'static>));

        debug_assert!(n >= 0);
        let n = n as usize;

        let rv = match activation_reason {
            x if x == ffi::VSActivationReason::arInitial as _ => {
                match filter.get_frame_initial(api, core, context, n) {
                    Ok(Some(frame)) => {
                        let ptr = frame.deref().deref() as *const _;
                        // The ownership is transferred to the caller.
                        mem::forget(frame);
                        ptr
                    }
                    Ok(None) => ptr::null(),
                    Err(err) => {
                        let mut buf = String::new();

                        buf += &format!("Error in Filter::get_frame_initial(): {}", err.cause());

                        push_backtrace(&mut buf, &err);

                        let buf = CString::new(buf.replace('\0', "\\0")).unwrap();
                        api.set_filter_error(buf.as_ptr(), frame_ctx);

                        ptr::null()
                    }
                }
            }
            x if x == ffi::VSActivationReason::arAllFramesReady as _ => {
                match filter.get_frame(api, core, context, n) {
                    Ok(frame) => {
                        let ptr = frame.deref().deref() as *const _;
                        // The ownership is transferred to the caller.
                        mem::forget(frame);
                        ptr
                    }
                    Err(err) => {
                        let mut buf = String::new();

                        buf += &format!("{}", err.cause());

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

/// Creates a new instance of the filter.
pub(crate) unsafe extern "system" fn create<F: FilterFunction>(
    in_: *const ffi::VSMap,
    out: *mut ffi::VSMap,
    user_data: *mut c_void,
    core: *mut ffi::VSCore,
    api: *const ffi::VSAPI,
) {
    let closure = move || {
        API::set(api);

        let args = MapRef::from_ptr(in_);
        let mut out = MapRefMut::from_ptr(out);
        let core = CoreRef::from_ptr(core);
        let data = Box::from_raw(user_data as *mut FilterFunctionData<F>);

        let filter = match data.filter_function.create(API::get_cached(), core, &args) {
            Ok(Some(filter)) => Some(Box::new(filter)),
            Ok(None) => None,
            Err(err) => {
                let mut buf = String::new();

                buf += &format!(
                    "Error in Filter::create() of {}: {}",
                    data.name.to_str().unwrap(),
                    err.cause()
                );

                push_backtrace(&mut buf, &err);

                out.set_error(&buf.replace('\0', "\\0")).unwrap();
                None
            }
        };

        if let Some(filter) = filter {
            API::get_cached().create_filter(
                in_,
                out.deref_mut().deref_mut(),
                data.name.as_ptr(),
                init,
                get_frame,
                Some(free),
                ffi::VSFilterMode::fmParallel,
                ffi::VSNodeFlags(0),
                Box::into_raw(filter) as *mut _,
                core.ptr(),
            );
        }

        mem::forget(data);
    };

    if panic::catch_unwind(closure).is_err() {
        // The `FilterFunction` might have been left in an inconsistent state, so we have to abort.
        process::abort();
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
pub unsafe fn call_register_func<F: FilterFunction>(
    register_func: *const c_void,
    plugin: *mut c_void,
    filter_function: F,
) {
    let register_func = *(&register_func as *const _ as *const ffi::VSRegisterFunction);

    let name_cstring = CString::new(filter_function.name())
        .expect("Couldn't convert the filter name to a CString");
    let args_cstring = CString::new(filter_function.args())
        .expect("Couldn't convert the filter args to a CString");

    let data = Box::new(FilterFunctionData {
        filter_function,
        name: name_cstring,
    });

    register_func(
        data.name.as_ptr(),
        args_cstring.as_ptr(),
        create::<F>,
        Box::into_raw(data) as _,
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
/// Following it is a list of values implementing `FilterFunction`, those are the filter functions
/// the plugin will export.
///
/// # Example
/// ```ignore
/// export_vapoursynth_plugin! {
///     Metadata {
///         identifier: "com.example.invert",
///         namespace: "invert",
///         name: "Invert Example Plugin",
///         read_only: true,
///     },
///     [SampleFilterFunction::new(), OtherFunction::new()]
/// }
/// ```
#[macro_export]
macro_rules! export_vapoursynth_plugin {
    ($metadata:expr, [$($filter:expr),*$(,)*]) => (
        use ::std::os::raw::c_void;

        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "system" fn VapourSynthPluginInit(
            config_func: *const c_void,
            register_func: *const c_void,
            plugin: *mut c_void,
        ) {
            use ::std::{panic, process};
            use $crate::plugins::ffi::{call_config_func, call_register_func};

            let closure = move || {
                call_config_func(config_func, plugin, $metadata);

                $(call_register_func(register_func, plugin, $filter);)*
            };

            if panic::catch_unwind(closure).is_err() {
                process::abort();
            }
        }
    )
}
