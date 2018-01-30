use std::ffi::{CStr, CString};
use std::fs::File;
use std::{mem, ptr};
use std::io::Read;
use std::path::Path;
use vapoursynth_sys as ffi;

use api::API;
use node::Node;
use vsscript::*;
use vsscript::errors::Result;

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

/// Contains two possible variants of arguments to `Environment::evaluate_script()`.
#[derive(Clone, Copy)]
enum EvaluateScriptArgs<'a> {
    /// Evaluate a script contained in the string.
    Script(&'a str),
    /// Evaluate a script contained in the file.
    File(&'a Path, EvalFlags),
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
    pub fn new() -> Result<Self> {
        maybe_initialize();

        let mut handle = unsafe { mem::uninitialized() };
        let rv = unsafe { call_vsscript!(ffi::vsscript_createScript(&mut handle)) };

        if rv != 0 {
            Err(unsafe {
                VSScriptError::from_environment(Self { handle }).into()
            })
        } else {
            Ok(Self { handle })
        }
    }

    /// Creates an invalid, null `Environment`.
    ///
    /// # Safety
    /// This function returns an invalid `Environment` which will behave incorrectly. The `handle`
    /// pointer must be set to a valid value before use.
    unsafe fn null() -> Self {
        Self {
            handle: ptr::null_mut(),
        }
    }

    /// Calls `vsscript_evaluateScript()`. Can be used to initialize a null `Environment`.
    ///
    /// # Safety
    /// The caller must ensure `vsscript_initialize()` has been called at least once.
    unsafe fn evaluate_script(mut self, args: EvaluateScriptArgs) -> Result<Self> {
        let (script, path, flags) = match args {
            EvaluateScriptArgs::Script(script) => (script.to_owned(), None, EvalFlags::Nothing),
            EvaluateScriptArgs::File(path, flags) => {
                let mut file = File::open(path).map_err(Error::FileOpen)?;
                let mut script = String::new();
                file.read_to_string(&mut script).map_err(Error::FileRead)?;

                // vsscript throws an error if it's not valid UTF-8 anyway.
                let path = path.to_str().ok_or(Error::PathInvalidUnicode)?;
                let path = CString::new(path)?;

                (script, Some(path), flags)
            }
        };

        let script = CString::new(script)?;

        let rv = call_vsscript!(ffi::vsscript_evaluateScript(
            &mut self.handle,
            script.as_ptr(),
            path.as_ref().map(|p| p.as_ptr()).unwrap_or(ptr::null()),
            flags.ffi_type(),
        ));

        if rv != 0 {
            Err(VSScriptError::from_environment(self).into())
        } else {
            Ok(self)
        }
    }

    /// Creates a script environment and evaluates a script contained in a string.
    pub fn from_script(script: &str) -> Result<Self> {
        maybe_initialize();

        unsafe { Self::evaluate_script(Self::null(), EvaluateScriptArgs::Script(script)) }
    }

    /// Creates a script environment and evaluates a script contained in a file.
    pub fn from_file<P: AsRef<Path>>(path: P, flags: EvalFlags) -> Result<Self> {
        maybe_initialize();

        unsafe {
            Self::evaluate_script(Self::null(), EvaluateScriptArgs::File(path.as_ref(), flags))
        }
    }

    /// Evaluates a script contained in a string.
    // TODO: somehow make this a method from &self?
    pub fn eval_script(self, script: &str) -> Result<Self> {
        unsafe { Self::evaluate_script(self, EvaluateScriptArgs::Script(script)) }
    }

    /// Evaluates a script contained in a file.
    // TODO: somehow make this a method from &self?
    pub fn eval_file<P: AsRef<Path>>(self, path: P, flags: EvalFlags) -> Result<Self> {
        unsafe { Self::evaluate_script(self, EvaluateScriptArgs::File(path.as_ref(), flags)) }
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

    /// Cancels a node set for output. The node will no longer be available to `get_output()`.
    ///
    /// If there's no node corresponding to the given `index`, `None` is returned.
    pub fn clear_output(&self, index: i32) -> Option<()> {
        let rv = unsafe { ffi::vsscript_clearOutput(self.handle, index) };
        if rv != 0 {
            None
        } else {
            Some(())
        }
    }

    /// Returns the error message from a script environment.
    ///
    /// # Safety
    /// This must be called when an error is known to be present.
    pub(crate) unsafe fn get_error(&self) -> &CStr {
        let ptr = ffi::vsscript_getError(self.handle);
        CStr::from_ptr(ptr)
    }
}
