use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::Read;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::ptr;
use std::ptr::NonNull;
use vapoursynth_sys as ffi;

use api::API;
use core::CoreRef;
use map::Map;
use node::Node;
use vsscript::errors::Result;
use vsscript::*;

/// VSScript file evaluation flags.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EvalFlags {
    Nothing,
    /// The working directory will be changed to the script's directory for the evaluation.
    SetWorkingDir,
}

impl EvalFlags {
    #[inline]
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
    handle: NonNull<ffi::VSScript>,
}

unsafe impl Send for Environment {}
unsafe impl Sync for Environment {}

impl Drop for Environment {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            ffi::vsscript_freeScript(self.handle.as_ptr());
        }
    }
}

impl Environment {
    /// Retrieves the VSScript error message.
    ///
    /// # Safety
    /// This function must only be called if an error is present.
    #[inline]
    unsafe fn error(&self) -> CString {
        let message = ffi::vsscript_getError(self.handle.as_ptr());
        CStr::from_ptr(message).to_owned()
    }

    /// Creates an empty script environment.
    ///
    /// Useful if it is necessary to set some variable in the script environment before evaluating
    /// any scripts.
    pub fn new() -> Result<Self> {
        maybe_initialize();

        let mut handle = ptr::null_mut();
        let rv = unsafe { call_vsscript!(ffi::vsscript_createScript(&mut handle)) };
        let environment = Self {
            handle: unsafe { NonNull::new_unchecked(handle) },
        };

        if rv != 0 {
            Err(VSScriptError::new(unsafe { environment.error() }).into())
        } else {
            Ok(environment)
        }
    }

    /// Calls `vsscript_evaluateScript()`.
    ///
    /// `self` is taken by a mutable reference mainly to ensure the atomicity of a call to
    /// `vsscript_evaluateScript()` (a function that could produce an error) and the following call
    /// to `vsscript_getError()`. If atomicity is not enforced, another thread could perform some
    /// operation between these two and clear or change the error message.
    fn evaluate_script(&mut self, args: EvaluateScriptArgs) -> Result<()> {
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

        let rv = unsafe {
            call_vsscript!(ffi::vsscript_evaluateScript(
                &mut self.handle.as_ptr(),
                script.as_ptr(),
                path.as_ref().map(|p| p.as_ptr()).unwrap_or(ptr::null()),
                flags.ffi_type(),
            ))
        };

        if rv != 0 {
            Err(VSScriptError::new(unsafe { self.error() }).into())
        } else {
            Ok(())
        }
    }

    /// Creates a script environment and evaluates a script contained in a string.
    #[inline]
    pub fn from_script(script: &str) -> Result<Self> {
        let mut environment = Self::new()?;
        environment.evaluate_script(EvaluateScriptArgs::Script(script))?;
        Ok(environment)
    }

    /// Creates a script environment and evaluates a script contained in a file.
    #[inline]
    pub fn from_file<P: AsRef<Path>>(path: P, flags: EvalFlags) -> Result<Self> {
        let mut environment = Self::new()?;
        environment.evaluate_script(EvaluateScriptArgs::File(path.as_ref(), flags))?;
        Ok(environment)
    }

    /// Evaluates a script contained in a string.
    #[inline]
    pub fn eval_script(&mut self, script: &str) -> Result<()> {
        self.evaluate_script(EvaluateScriptArgs::Script(script))
    }

    /// Evaluates a script contained in a file.
    #[inline]
    pub fn eval_file<P: AsRef<Path>>(&mut self, path: P, flags: EvalFlags) -> Result<()> {
        self.evaluate_script(EvaluateScriptArgs::File(path.as_ref(), flags))
    }

    /// Clears the script environment.
    #[inline]
    pub fn clear(&self) {
        unsafe {
            ffi::vsscript_clearEnvironment(self.handle.as_ptr());
        }
    }

    /// Retrieves a node from the script environment. A node in the script must have been marked
    /// for output with the requested index.
    #[cfg(all(
        not(feature = "gte-vsscript-api-31"),
        feature = "vapoursynth-functions"
    ))]
    #[inline]
    pub fn get_output(&self, index: i32) -> Result<Node> {
        // Node needs the API.
        API::get().ok_or(Error::NoAPI)?;

        let node_handle = unsafe { ffi::vsscript_getOutput(self.handle.as_ptr(), index) };
        if node_handle.is_null() {
            Err(Error::NoOutput)
        } else {
            Ok(unsafe { Node::from_ptr(node_handle) })
        }
    }

    /// Retrieves a node from the script environment. A node in the script must have been marked
    /// for output with the requested index. The second node, if any, contains the alpha clip.
    #[cfg(all(
        feature = "gte-vsscript-api-31",
        any(
            feature = "vapoursynth-functions",
            feature = "gte-vsscript-api-32"
        )
    ))]
    #[inline]
    pub fn get_output(&self, index: i32) -> Result<(Node, Option<Node>)> {
        // Node needs the API.
        API::get().ok_or(Error::NoAPI)?;

        let mut alpha_handle = ptr::null_mut();
        let node_handle =
            unsafe { ffi::vsscript_getOutput2(self.handle.as_ptr(), index, &mut alpha_handle) };

        if node_handle.is_null() {
            return Err(Error::NoOutput);
        }

        let node = unsafe { Node::from_ptr(node_handle) };
        let alpha_node = unsafe { alpha_handle.as_mut().map(|p| Node::from_ptr(p)) };

        Ok((node, alpha_node))
    }

    /// Cancels a node set for output. The node will no longer be available to `get_output()`.
    #[inline]
    pub fn clear_output(&self, index: i32) -> Result<()> {
        let rv = unsafe { ffi::vsscript_clearOutput(self.handle.as_ptr(), index) };
        if rv != 0 {
            Err(Error::NoOutput)
        } else {
            Ok(())
        }
    }

    /// Retrieves the VapourSynth core that was created in the script environment. If a VapourSynth
    /// core has not been created yet, it will be created now, with the default options.
    #[cfg(any(
        feature = "vapoursynth-functions",
        feature = "gte-vsscript-api-32"
    ))]
    pub fn get_core(&self) -> Result<CoreRef> {
        // CoreRef needs the API.
        API::get().ok_or(Error::NoAPI)?;

        let ptr = unsafe { ffi::vsscript_getCore(self.handle.as_ptr()) };
        if ptr.is_null() {
            Err(Error::NoCore)
        } else {
            Ok(unsafe { CoreRef::from_ptr(ptr) })
        }
    }

    /// Retrieves a variable from the script environment.
    pub fn get_variable(&self, name: &str, map: &mut Map) -> Result<()> {
        let name = CString::new(name)?;
        let rv = unsafe {
            ffi::vsscript_getVariable(self.handle.as_ptr(), name.as_ptr(), map.deref_mut())
        };
        if rv != 0 {
            Err(Error::NoSuchVariable)
        } else {
            Ok(())
        }
    }

    /// Sets variables in the script environment.
    pub fn set_variables(&self, variables: &Map) -> Result<()> {
        let rv = unsafe { ffi::vsscript_setVariable(self.handle.as_ptr(), variables.deref()) };
        if rv != 0 {
            Err(Error::NoSuchVariable)
        } else {
            Ok(())
        }
    }

    /// Deletes a variable from the script environment.
    pub fn clear_variable(&self, name: &str) -> Result<()> {
        let name = CString::new(name)?;
        let rv = unsafe { ffi::vsscript_clearVariable(self.handle.as_ptr(), name.as_ptr()) };
        if rv != 0 {
            Err(Error::NoSuchVariable)
        } else {
            Ok(())
        }
    }
}
