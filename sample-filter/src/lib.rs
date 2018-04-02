///! A sample VapourSynth plugin.
#[macro_use]
extern crate failure;
#[macro_use]
extern crate vapoursynth;

use std::slice;

use failure::Error;
use vapoursynth::prelude::*;
use vapoursynth::plugins::*;
use vapoursynth::core::CoreRef;
use vapoursynth::video_info::VideoInfo;

struct Invert {
    source: Node,
}

impl Filter for Invert {
    fn name() -> &'static str {
        "Invert"
    }

    fn args() -> &'static str {
        "clip:clip"
    }

    fn create(_api: API, _core: CoreRef, args: &Map) -> Result<Self, Error> {
        let source = args.get_node("clip").unwrap();
        Ok(Invert { source })
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
        core: CoreRef,
        context: FrameContext,
        n: usize,
    ) -> Result<FrameRef, Error> {
        let frame = self.source
            .get_frame_filter(context, n)
            .ok_or(format_err!("Couldn't get the source frame"))?;

        if frame.format().sample_type() == SampleType::Float {
            bail!("Floating point formats are not supported");
        }

        let mut frame = FrameRefMut::copy_of(core, &frame);

        for plane in 0..frame.format().plane_count() {
            for row in 0..frame.height(plane) {
                assert_eq!(frame.format().sample_type(), SampleType::Integer);

                let bits_per_sample = frame.format().bits_per_sample();
                let bytes_per_sample = frame.format().bytes_per_sample();

                let data = frame.data_row_mut(plane, row);

                match bytes_per_sample {
                    1 => for pixel in data {
                        *pixel = 255 - *pixel;
                    },
                    2 => {
                        let row = unsafe {
                            slice::from_raw_parts_mut(
                                data.as_mut_ptr() as *mut u16,
                                data.len() / bytes_per_sample as usize,
                            )
                        };
                        for pixel in row {
                            *pixel = ((1u64 << bits_per_sample) - 1) as u16 - *pixel;
                        }
                    }
                    4 => {
                        let row = unsafe {
                            slice::from_raw_parts_mut(
                                data.as_mut_ptr() as *mut u32,
                                data.len() / bytes_per_sample as usize,
                            )
                        };
                        for pixel in row {
                            *pixel = ((1u64 << bits_per_sample) - 1) as u32 - *pixel;
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }

        Ok(frame.into())
    }
}

export_vapoursynth_plugin! {
    Metadata {
        identifier: "com.example.vapoursynth-rs",
        namespace: "vapoursynth_rs",
        name: "Example vapoursynth-rs Plugin",
        read_only: true,
    },
    [Invert]
}
