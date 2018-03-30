///! A sample VapourSynth plugin.
#[macro_use]
extern crate failure;
#[macro_use]
extern crate vapoursynth;

use failure::Error;
use vapoursynth::prelude::*;
use vapoursynth::plugins::*;
use vapoursynth::core::CoreRef;
use vapoursynth::video_info::VideoInfo;

struct SampleFilter {
    source: Node,
}

impl Filter for SampleFilter {
    fn name() -> &'static str {
        "SampleName"
    }

    fn args() -> &'static str {
        "clip:clip"
    }

    fn create(_api: API, _core: CoreRef, args: &Map) -> Result<Self, Error> {
        let source = args.get_node("clip").unwrap();
        Ok(SampleFilter { source })
    }

    fn video_info(&self, _api: API, _core: CoreRef) -> Vec<VideoInfo> {
        vec![self.source.info()]
    }

    fn get_frame_initial(
        &self,
        _api: API,
        _core: CoreRef,
        context: FrameContext,
        n: usize,
    ) -> Result<(), Error> {
        self.source.request_frame_filter(context, n);
        Ok(())
    }

    fn get_frame(
        &self,
        _api: API,
        _core: CoreRef,
        context: FrameContext,
        n: usize,
    ) -> Result<Frame, Error> {
        self.source
            .get_frame_filter(context, n)
            .ok_or(format_err!("Couldn't get the source frame"))
    }
}

export_vapoursynth_plugin! {
    Metadata {
        identifier: "com.example.invert",
        namespace: "invert",
        name: "Invert Example Plugin",
        read_only: true,
    },
    [SampleFilter]
}
