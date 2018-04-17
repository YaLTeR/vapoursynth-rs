use std::os::raw::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VSFrameRef {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VSNodeRef {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VSCore {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VSPlugin {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VSNode {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VSFuncRef {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VSMap {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VSFrameContext {
    _unused: [u8; 0],
}
#[cfg_attr(feature = "cargo-clippy", allow(unreadable_literal))]
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VSColorFamily {
    cmGray = 1000000,
    cmRGB = 2000000,
    cmYUV = 3000000,
    cmYCoCg = 4000000,
    cmCompat = 9000000,
}
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VSSampleType {
    stInteger = 0,
    stFloat = 1,
}
#[cfg_attr(feature = "cargo-clippy", allow(unreadable_literal))]
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VSPresetFormat {
    pfNone = 0,
    pfGray8 = 1000010,
    pfGray16 = 1000011,
    pfGrayH = 1000012,
    pfGrayS = 1000013,
    pfYUV420P8 = 3000010,
    pfYUV422P8 = 3000011,
    pfYUV444P8 = 3000012,
    pfYUV410P8 = 3000013,
    pfYUV411P8 = 3000014,
    pfYUV440P8 = 3000015,
    pfYUV420P9 = 3000016,
    pfYUV422P9 = 3000017,
    pfYUV444P9 = 3000018,
    pfYUV420P10 = 3000019,
    pfYUV422P10 = 3000020,
    pfYUV444P10 = 3000021,
    pfYUV420P16 = 3000022,
    pfYUV422P16 = 3000023,
    pfYUV444P16 = 3000024,
    pfYUV444PH = 3000025,
    pfYUV444PS = 3000026,
    pfYUV420P12 = 3000027,
    pfYUV422P12 = 3000028,
    pfYUV444P12 = 3000029,
    pfYUV420P14 = 3000030,
    pfYUV422P14 = 3000031,
    pfYUV444P14 = 3000032,
    pfRGB24 = 2000010,
    pfRGB27 = 2000011,
    pfRGB30 = 2000012,
    pfRGB48 = 2000013,
    pfRGBH = 2000014,
    pfRGBS = 2000015,
    pfCompatBGR32 = 9000010,
    pfCompatYUY2 = 9000011,
}
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VSFilterMode {
    fmParallel = 100,
    fmParallelRequests = 200,
    fmUnordered = 300,
    fmSerial = 400,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VSFormat {
    pub name: [c_char; 32usize],
    pub id: c_int,
    pub colorFamily: c_int,
    pub sampleType: c_int,
    pub bitsPerSample: c_int,
    pub bytesPerSample: c_int,
    pub subSamplingW: c_int,
    pub subSamplingH: c_int,
    pub numPlanes: c_int,
}
pub const VSNodeFlags_nfNoCache: VSNodeFlags = VSNodeFlags(1);
pub const VSNodeFlags_nfIsCache: VSNodeFlags = VSNodeFlags(2);
#[cfg(feature = "gte-vapoursynth-api-33")]
pub const VSNodeFlags_nfMakeLinear: VSNodeFlags = VSNodeFlags(4);
impl ::std::ops::BitOr<VSNodeFlags> for VSNodeFlags {
    type Output = Self;
    #[inline]
    fn bitor(self, other: Self) -> Self {
        VSNodeFlags(self.0 | other.0)
    }
}
impl ::std::ops::BitOrAssign for VSNodeFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: VSNodeFlags) {
        self.0 |= rhs.0;
    }
}
impl ::std::ops::BitAnd<VSNodeFlags> for VSNodeFlags {
    type Output = Self;
    #[inline]
    fn bitand(self, other: Self) -> Self {
        VSNodeFlags(self.0 & other.0)
    }
}
impl ::std::ops::BitAndAssign for VSNodeFlags {
    #[inline]
    fn bitand_assign(&mut self, rhs: VSNodeFlags) {
        self.0 &= rhs.0;
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct VSNodeFlags(pub c_int);
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VSPropTypes {
    ptUnset = 117,
    ptInt = 105,
    ptFloat = 102,
    ptData = 115,
    ptNode = 99,
    ptFrame = 118,
    ptFunction = 109,
}
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VSGetPropErrors {
    peUnset = 1,
    peType = 2,
    peIndex = 4,
}
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VSPropAppendMode {
    paReplace = 0,
    paAppend = 1,
    paTouch = 2,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VSCoreInfo {
    pub versionString: *const c_char,
    pub core: c_int,
    pub api: c_int,
    pub numThreads: c_int,
    pub maxFramebufferSize: i64,
    pub usedFramebufferSize: i64,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VSVideoInfo {
    pub format: *const VSFormat,
    pub fpsNum: i64,
    pub fpsDen: i64,
    pub width: c_int,
    pub height: c_int,
    pub numFrames: c_int,
    pub flags: c_int,
}
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VSActivationReason {
    arInitial = 0,
    arFrameReady = 1,
    arAllFramesReady = 2,
    arError = -1,
}
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VSMessageType {
    mtDebug = 0,
    mtWarning = 1,
    mtCritical = 2,
    mtFatal = 3,
}
pub type VSPublicFunction = unsafe extern "system" fn(
    in_: *const VSMap,
    out: *mut VSMap,
    userData: *mut c_void,
    core: *mut VSCore,
    vsapi: *const VSAPI,
);
pub type VSRegisterFunction = unsafe extern "system" fn(
    name: *const c_char,
    args: *const c_char,
    argsFunc: VSPublicFunction,
    functionData: *mut c_void,
    plugin: *mut VSPlugin,
);
pub type VSConfigPlugin = unsafe extern "system" fn(
    identifier: *const c_char,
    defaultNamespace: *const c_char,
    name: *const c_char,
    apiVersion: c_int,
    readonly: c_int,
    plugin: *mut VSPlugin,
);
pub type VSInitPlugin = Option<
    unsafe extern "system" fn(
        configFunc: VSConfigPlugin,
        registerFunc: VSRegisterFunction,
        plugin: *mut VSPlugin,
    ),
>;
pub type VSFreeFuncData = Option<unsafe extern "system" fn(userData: *mut c_void)>;
pub type VSFilterInit = unsafe extern "system" fn(
    in_: *mut VSMap,
    out: *mut VSMap,
    instanceData: *mut *mut c_void,
    node: *mut VSNode,
    core: *mut VSCore,
    vsapi: *const VSAPI,
);
pub type VSFilterGetFrame = unsafe extern "system" fn(
    n: c_int,
    activationReason: c_int,
    instanceData: *mut *mut c_void,
    frameData: *mut *mut c_void,
    frameCtx: *mut VSFrameContext,
    core: *mut VSCore,
    vsapi: *const VSAPI,
) -> *const VSFrameRef;
pub type VSFilterFree = Option<
    unsafe extern "system" fn(instanceData: *mut c_void, core: *mut VSCore, vsapi: *const VSAPI),
>;
pub type VSFrameDoneCallback = Option<
    unsafe extern "system" fn(
        userData: *mut c_void,
        f: *const VSFrameRef,
        n: c_int,
        arg1: *mut VSNodeRef,
        errorMsg: *const c_char,
    ),
>;
pub type VSMessageHandler =
    Option<unsafe extern "system" fn(msgType: c_int, msg: *const c_char, userData: *mut c_void)>;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct VSAPI {
    pub createCore: unsafe extern "system" fn(threads: c_int) -> *mut VSCore,
    pub freeCore: unsafe extern "system" fn(core: *mut VSCore),
    pub getCoreInfo: unsafe extern "system" fn(core: *mut VSCore) -> *const VSCoreInfo,
    pub cloneFrameRef: unsafe extern "system" fn(f: *const VSFrameRef) -> *const VSFrameRef,
    pub cloneNodeRef: unsafe extern "system" fn(node: *mut VSNodeRef) -> *mut VSNodeRef,
    pub cloneFuncRef: unsafe extern "system" fn(f: *mut VSFuncRef) -> *mut VSFuncRef,
    pub freeFrame: unsafe extern "system" fn(f: *const VSFrameRef),
    pub freeNode: unsafe extern "system" fn(node: *mut VSNodeRef),
    pub freeFunc: unsafe extern "system" fn(f: *mut VSFuncRef),
    pub newVideoFrame: unsafe extern "system" fn(
        format: *const VSFormat,
        width: c_int,
        height: c_int,
        propSrc: *const VSFrameRef,
        core: *mut VSCore,
    ) -> *mut VSFrameRef,
    pub copyFrame:
        unsafe extern "system" fn(f: *const VSFrameRef, core: *mut VSCore) -> *mut VSFrameRef,
    pub copyFrameProps:
        unsafe extern "system" fn(src: *const VSFrameRef, dst: *mut VSFrameRef, core: *mut VSCore),
    pub registerFunction: unsafe extern "system" fn(
        name: *const c_char,
        args: *const c_char,
        argsFunc: VSPublicFunction,
        functionData: *mut c_void,
        plugin: *mut VSPlugin,
    ),
    pub getPluginById:
        unsafe extern "system" fn(identifier: *const c_char, core: *mut VSCore) -> *mut VSPlugin,
    pub getPluginByNs:
        unsafe extern "system" fn(ns: *const c_char, core: *mut VSCore) -> *mut VSPlugin,
    pub getPlugins: unsafe extern "system" fn(core: *mut VSCore) -> *mut VSMap,
    pub getFunctions: unsafe extern "system" fn(plugin: *mut VSPlugin) -> *mut VSMap,
    #[cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
    pub createFilter: unsafe extern "system" fn(
        in_: *const VSMap,
        out: *mut VSMap,
        name: *const c_char,
        init: VSFilterInit,
        getFrame: VSFilterGetFrame,
        free: VSFilterFree,
        filterMode: c_int,
        flags: c_int,
        instanceData: *mut c_void,
        core: *mut VSCore,
    ),
    pub setError: unsafe extern "system" fn(map: *mut VSMap, errorMessage: *const c_char),
    pub getError: unsafe extern "system" fn(map: *const VSMap) -> *const c_char,
    pub setFilterError:
        unsafe extern "system" fn(errorMessage: *const c_char, frameCtx: *mut VSFrameContext),
    pub invoke:
        unsafe extern "system" fn(plugin: *mut VSPlugin, name: *const c_char, args: *const VSMap)
            -> *mut VSMap,
    pub getFormatPreset: unsafe extern "system" fn(id: c_int, core: *mut VSCore) -> *const VSFormat,
    pub registerFormat: unsafe extern "system" fn(
        colorFamily: c_int,
        sampleType: c_int,
        bitsPerSample: c_int,
        subSamplingW: c_int,
        subSamplingH: c_int,
        core: *mut VSCore,
    ) -> *const VSFormat,
    pub getFrame: unsafe extern "system" fn(
        n: c_int,
        node: *mut VSNodeRef,
        errorMsg: *mut c_char,
        bufSize: c_int,
    ) -> *const VSFrameRef,
    pub getFrameAsync: unsafe extern "system" fn(
        n: c_int,
        node: *mut VSNodeRef,
        callback: VSFrameDoneCallback,
        userData: *mut c_void,
    ),
    pub getFrameFilter:
        unsafe extern "system" fn(n: c_int, node: *mut VSNodeRef, frameCtx: *mut VSFrameContext)
            -> *const VSFrameRef,
    pub requestFrameFilter:
        unsafe extern "system" fn(n: c_int, node: *mut VSNodeRef, frameCtx: *mut VSFrameContext),
    pub queryCompletedFrame: unsafe extern "system" fn(
        node: *mut *mut VSNodeRef,
        n: *mut c_int,
        frameCtx: *mut VSFrameContext,
    ),
    pub releaseFrameEarly:
        unsafe extern "system" fn(node: *mut VSNodeRef, n: c_int, frameCtx: *mut VSFrameContext),
    pub getStride: unsafe extern "system" fn(f: *const VSFrameRef, plane: c_int) -> c_int,
    pub getReadPtr: unsafe extern "system" fn(f: *const VSFrameRef, plane: c_int) -> *const u8,
    pub getWritePtr: unsafe extern "system" fn(f: *mut VSFrameRef, plane: c_int) -> *mut u8,
    pub createFunc: unsafe extern "system" fn(
        func: VSPublicFunction,
        userData: *mut c_void,
        free: VSFreeFuncData,
        core: *mut VSCore,
        vsapi: *const VSAPI,
    ) -> *mut VSFuncRef,
    pub callFunc: unsafe extern "system" fn(
        func: *mut VSFuncRef,
        in_: *const VSMap,
        out: *mut VSMap,
        core: *mut VSCore,
        vsapi: *const VSAPI,
    ),
    pub createMap: unsafe extern "system" fn() -> *mut VSMap,
    pub freeMap: unsafe extern "system" fn(map: *mut VSMap),
    pub clearMap: unsafe extern "system" fn(map: *mut VSMap),
    pub getVideoInfo: unsafe extern "system" fn(node: *mut VSNodeRef) -> *const VSVideoInfo,
    pub setVideoInfo:
        unsafe extern "system" fn(vi: *const VSVideoInfo, numOutputs: c_int, node: *mut VSNode),
    pub getFrameFormat: unsafe extern "system" fn(f: *const VSFrameRef) -> *const VSFormat,
    pub getFrameWidth: unsafe extern "system" fn(f: *const VSFrameRef, plane: c_int) -> c_int,
    pub getFrameHeight: unsafe extern "system" fn(f: *const VSFrameRef, plane: c_int) -> c_int,
    pub getFramePropsRO: unsafe extern "system" fn(f: *const VSFrameRef) -> *const VSMap,
    pub getFramePropsRW: unsafe extern "system" fn(f: *mut VSFrameRef) -> *mut VSMap,
    pub propNumKeys: unsafe extern "system" fn(map: *const VSMap) -> c_int,
    pub propGetKey: unsafe extern "system" fn(map: *const VSMap, index: c_int) -> *const c_char,
    pub propNumElements: unsafe extern "system" fn(map: *const VSMap, key: *const c_char) -> c_int,
    pub propGetType: unsafe extern "system" fn(map: *const VSMap, key: *const c_char) -> c_char,
    pub propGetInt: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut c_int,
    ) -> i64,
    pub propGetFloat: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut c_int,
    ) -> f64,
    pub propGetData: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut c_int,
    ) -> *const c_char,
    pub propGetDataSize: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut c_int,
    ) -> c_int,
    pub propGetNode: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut c_int,
    ) -> *mut VSNodeRef,
    pub propGetFrame: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut c_int,
    ) -> *const VSFrameRef,
    pub propGetFunc: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut c_int,
    ) -> *mut VSFuncRef,
    pub propDeleteKey: unsafe extern "system" fn(map: *mut VSMap, key: *const c_char) -> c_int,
    pub propSetInt:
        unsafe extern "system" fn(map: *mut VSMap, key: *const c_char, i: i64, append: c_int)
            -> c_int,
    pub propSetFloat:
        unsafe extern "system" fn(map: *mut VSMap, key: *const c_char, d: f64, append: c_int)
            -> c_int,
    pub propSetData: unsafe extern "system" fn(
        map: *mut VSMap,
        key: *const c_char,
        data: *const c_char,
        size: c_int,
        append: c_int,
    ) -> c_int,
    pub propSetNode: unsafe extern "system" fn(
        map: *mut VSMap,
        key: *const c_char,
        node: *mut VSNodeRef,
        append: c_int,
    ) -> c_int,
    pub propSetFrame: unsafe extern "system" fn(
        map: *mut VSMap,
        key: *const c_char,
        f: *const VSFrameRef,
        append: c_int,
    ) -> c_int,
    pub propSetFunc: unsafe extern "system" fn(
        map: *mut VSMap,
        key: *const c_char,
        func: *mut VSFuncRef,
        append: c_int,
    ) -> c_int,
    pub setMaxCacheSize: unsafe extern "system" fn(bytes: i64, core: *mut VSCore) -> i64,
    pub getOutputIndex: unsafe extern "system" fn(frameCtx: *mut VSFrameContext) -> c_int,
    pub newVideoFrame2: unsafe extern "system" fn(
        format: *const VSFormat,
        width: c_int,
        height: c_int,
        planeSrc: *mut *const VSFrameRef,
        planes: *const c_int,
        propSrc: *const VSFrameRef,
        core: *mut VSCore,
    ) -> *mut VSFrameRef,
    pub setMessageHandler:
        unsafe extern "system" fn(handler: VSMessageHandler, userData: *mut c_void),
    pub setThreadCount: unsafe extern "system" fn(threads: c_int, core: *mut VSCore) -> c_int,
    pub getPluginPath: unsafe extern "system" fn(plugin: *const VSPlugin) -> *const c_char,

    #[cfg(feature = "gte-vapoursynth-api-31")]
    pub propGetIntArray:
        unsafe extern "system" fn(map: *const VSMap, key: *const c_char, error: *mut c_int)
            -> *const i64,
    #[cfg(feature = "gte-vapoursynth-api-31")]
    pub propGetFloatArray:
        unsafe extern "system" fn(map: *const VSMap, key: *const c_char, error: *mut c_int)
            -> *const f64,
    #[cfg(feature = "gte-vapoursynth-api-31")]
    pub propSetIntArray:
        unsafe extern "system" fn(map: *mut VSMap, key: *const c_char, i: *const i64, size: c_int)
            -> c_int,
    #[cfg(feature = "gte-vapoursynth-api-31")]
    pub propSetFloatArray:
        unsafe extern "system" fn(map: *mut VSMap, key: *const c_char, d: *const f64, size: c_int)
            -> c_int,
    #[cfg(feature = "gte-vapoursynth-api-34")]
    pub logMessage: unsafe extern "system" fn(msgType: c_int, msg: *const c_char),
}

#[cfg(feature = "vapoursynth-functions")]
extern "system" {
    pub fn getVapourSynthAPI(version: c_int) -> *const VSAPI;
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VSScript {
    _unused: [u8; 0],
}
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VSEvalFlags {
    efSetWorkingDir = 1,
}

#[cfg(feature = "vsscript-functions")]
extern "system" {
    #[cfg(feature = "gte-vsscript-api-31")]
    pub fn vsscript_getApiVersion() -> c_int;
    pub fn vsscript_init() -> c_int;
    pub fn vsscript_finalize() -> c_int;
    pub fn vsscript_evaluateScript(
        handle: *mut *mut VSScript,
        script: *const c_char,
        scriptFilename: *const c_char,
        flags: c_int,
    ) -> c_int;
    pub fn vsscript_evaluateFile(
        handle: *mut *mut VSScript,
        scriptFilename: *const c_char,
        flags: c_int,
    ) -> c_int;
    pub fn vsscript_createScript(handle: *mut *mut VSScript) -> c_int;
    pub fn vsscript_freeScript(handle: *mut VSScript);
    pub fn vsscript_getError(handle: *mut VSScript) -> *const c_char;
    pub fn vsscript_getOutput(handle: *mut VSScript, index: c_int) -> *mut VSNodeRef;
    #[cfg(feature = "gte-vsscript-api-31")]
    pub fn vsscript_getOutput2(
        handle: *mut VSScript,
        index: c_int,
        alpha: *mut *mut VSNodeRef,
    ) -> *mut VSNodeRef;
    pub fn vsscript_clearOutput(handle: *mut VSScript, index: c_int) -> c_int;
    pub fn vsscript_getCore(handle: *mut VSScript) -> *mut VSCore;
    pub fn vsscript_getVSApi() -> *const VSAPI;
    #[cfg(feature = "gte-vsscript-api-32")]
    pub fn vsscript_getVSApi2(version: c_int) -> *const VSAPI;
    pub fn vsscript_getVariable(
        handle: *mut VSScript,
        name: *const c_char,
        dst: *mut VSMap,
    ) -> c_int;
    pub fn vsscript_setVariable(handle: *mut VSScript, vars: *const VSMap) -> c_int;
    pub fn vsscript_clearVariable(handle: *mut VSScript, name: *const c_char) -> c_int;
    pub fn vsscript_clearEnvironment(handle: *mut VSScript);
}
