use failure::{self, Fail, ResultExt};
use std::ffi::{CStr, CString};
use std::fs::File;
use std::{fmt, mem, ptr};
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};
use vapoursynth_sys as ffi;

use api::API;
use node::Node;

/// True if `vsscript_init()` was already called.
static VSSCRIPT_INITIALIZED: AtomicBool = ATOMIC_BOOL_INIT;

/// Ensures `vsscript_init()` has been called at least once.
fn maybe_initialize() {
    if !VSSCRIPT_INITIALIZED.swap(true, Ordering::Relaxed) {
        unsafe {
            ffi::vsscript_init();
        }
    }
}

/// A container for a VSScript error.
#[derive(Fail, Debug)]
pub struct Error(Environment);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = unsafe { CStr::from_ptr(ffi::vsscript_getError(self.0.handle)) };

        write!(f, "{}", message.to_string_lossy())
    }
}

/// VSScript file evaluation flags.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EvalFlags {
    Nothing,
    /// The working directory will be changed to the script's directory for the evaluation.
    SetWorkingDir,
}

impl EvalFlags {
    fn ffi_type(self) -> ::std::os::raw::c_int {
        match self {
            EvalFlags::Nothing => 0,
            EvalFlags::SetWorkingDir => ffi::VSEvalFlags::efSetWorkingDir as _,
        }
    }
}

/// A wrapper for the VSScript environment.
#[derive(Debug)]
pub struct Environment {
    handle: *mut ffi::VSScript,
}

unsafe impl Send for Environment {}
unsafe impl Sync for Environment {}

impl Drop for Environment {
    fn drop(&mut self) {
        unsafe {
            ffi::vsscript_freeScript(self.handle);
        }
    }
}

impl Environment {
    /// Creates an empty script environment.
    ///
    /// Useful if it is necessary to set some variable in the script environment before evaluating
    /// any scripts.
    pub fn new() -> Result<Self, Error> {
        maybe_initialize();

        let mut handle = unsafe { mem::uninitialized() };
        let rv = unsafe { ffi::vsscript_createScript(&mut handle) };

        if rv != 0 {
            Err(Error(Self { handle }))
        } else {
            Ok(Self { handle })
        }
    }

    /// Creates a script environment and evaluates a script contained in a string.
    pub fn from_script(script: &str) -> Result<Self, failure::Error> {
        let script = CString::new(script).context("Couldn't convert script into a CString")?;

        maybe_initialize();

        let mut handle = ptr::null_mut();
        let rv =
            unsafe { ffi::vsscript_evaluateScript(&mut handle, script.as_ptr(), ptr::null(), 0) };

        if rv != 0 {
            Err(
                Error(Self { handle })
                    .context("Error in vsscript_evaluateScript()")
                    .into(),
            )
        } else {
            Ok(Self { handle })
        }
    }

    /// Creates a script environment and evaluates a script contained in a file.
    pub fn from_file<P: AsRef<Path>>(path: P, flags: EvalFlags) -> Result<Self, failure::Error> {
        let mut file = File::open(path.as_ref()).context("Couldn't open the file")?;
        let mut script = String::new();
        file.read_to_string(&mut script)
            .context("Couldn't read the file")?;
        drop(file);

        let script = CString::new(script).context("Couldn't convert script into a CString")?;

        // vsscript throws an error if it's not valid UTF-8 anyway.
        let path = path.as_ref()
            .to_str()
            .ok_or(format_err!("Path was not valid unicode"))?;
        let path = CString::new(path).context("Couldn't convert path into a CString")?;

        maybe_initialize();

        let mut handle = ptr::null_mut();
        let rv = unsafe {
            ffi::vsscript_evaluateScript(
                &mut handle,
                script.as_ptr(),
                path.as_ptr(),
                flags.ffi_type(),
            )
        };

        if rv != 0 {
            Err(
                Error(Self { handle })
                    .context("Error in vsscript_evaluateScript()")
                    .into(),
            )
        } else {
            Ok(Self { handle })
        }
    }

    /// Clears the script environment.
    pub fn clear(&self) {
        unsafe {
            ffi::vsscript_clearEnvironment(self.handle);
        }
    }

    /// Retrieves a node from the script environment. A node in the script must have been marked
    /// for output with the requested index.
    ///
    /// If there's no node corresponding to the given `index`, `None` is returned.
    pub fn get_output(&self, api: API, index: i32) -> Option<Node> {
        let node_handle = unsafe { ffi::vsscript_getOutput(self.handle, index) };
        if node_handle.is_null() {
            None
        } else {
            Some(unsafe { Node::new(api, node_handle) })
        }
    }
}
