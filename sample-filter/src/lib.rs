///! A sample VapourSynth plugin.
#[macro_use]
extern crate failure;
extern crate rand;
#[macro_use]
extern crate vapoursynth;

use std::{ptr, slice};

use failure::Error;
use rand::Rng;
use vapoursynth::prelude::*;
use vapoursynth::plugins::*;
use vapoursynth::core::CoreRef;
use vapoursynth::format::FormatID;
use vapoursynth::node::Flags;
use vapoursynth::video_info::{Framerate, Resolution, VideoInfo};

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
    ) -> Result<Option<FrameRef>, Error> {
        self.source.request_frame_filter(context, n);
        Ok(None)
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
                        let data = unsafe {
                            slice::from_raw_parts_mut(
                                data.as_mut_ptr() as *mut u16,
                                data.len() / bytes_per_sample as usize,
                            )
                        };
                        for pixel in data {
                            *pixel = ((1u64 << bits_per_sample) - 1) as u16 - *pixel;
                        }
                    }
                    4 => {
                        let data = unsafe {
                            slice::from_raw_parts_mut(
                                data.as_mut_ptr() as *mut u32,
                                data.len() / bytes_per_sample as usize,
                            )
                        };
                        for pixel in data {
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

struct RandomNoise {
    format_id: FormatID,
    resolution: Resolution,
    framerate: Framerate,
    length: usize,
}

impl Filter for RandomNoise {
    fn name() -> &'static str {
        "RandomNoise"
    }

    fn args() -> &'static str {
        "width:int;height:int;format:int;length:int;fpsnum:int;fpsden:int"
    }

    fn create(_api: API, core: CoreRef, args: &Map) -> Result<Self, Error> {
        let format_id = (args.get_int("format").unwrap() as i32).into();
        let format = core.get_format(format_id)
            .ok_or(format_err!("No such format"))?;

        if format.sample_type() == SampleType::Float {
            bail!("Floating point formats are not supported");
        }

        let width = args.get_int("width").unwrap();
        if width <= 0 || width > i32::max_value() as i64 {
            bail!("Invalid width");
        }
        let width = width as usize;

        let height = args.get_int("height").unwrap();
        if height <= 0 || height > i32::max_value() as i64 {
            bail!("Invalid height");
        }
        let height = height as usize;

        let length = args.get_int("length").unwrap();
        if length <= 0 || length > i32::max_value() as i64 {
            bail!("Invalid length");
        }
        let length = length as usize;

        let fpsnum = args.get_int("fpsnum").unwrap();
        if fpsnum <= 0 {
            bail!("Invalid fpsnum");
        }
        let fpsnum = fpsnum as u64;

        let fpsden = args.get_int("fpsden").unwrap();
        if fpsden <= 0 {
            bail!("Invalid fpsden");
        }
        let fpsden = fpsden as u64;

        Ok(RandomNoise {
            format_id,
            resolution: Resolution { width, height },
            framerate: Framerate {
                numerator: fpsnum,
                denominator: fpsden,
            },
            length,
        })
    }

    fn video_info<'a>(&'a self, _api: API, core: CoreRef<'a>) -> Vec<VideoInfo> {
        vec![
            VideoInfo {
                format: core.get_format(self.format_id).unwrap().into(),
                resolution: self.resolution.into(),
                framerate: self.framerate.into(),
                num_frames: self.length.into(),
                flags: Flags::empty(),
            },
        ]
    }

    fn get_frame_initial(
        &self,
        _api: API,
        core: CoreRef,
        _context: FrameContext,
        _n: usize,
    ) -> Result<Option<FrameRef>, Error> {
        let format = core.get_format(self.format_id).unwrap();
        let mut frame =
            unsafe { FrameRefMut::new_uninitialized(core, None, format, self.resolution) };

        for plane in 0..frame.format().plane_count() {
            for row in 0..frame.height(plane) {
                assert_eq!(frame.format().sample_type(), SampleType::Integer);

                let bytes_per_sample = frame.format().bytes_per_sample();

                let mut rng = rand::thread_rng();
                let data = frame.data_row_mut(plane, row);

                match bytes_per_sample {
                    1 => for col in 0..data.len() {
                        unsafe {
                            ptr::write(data.as_mut_ptr().offset(col as isize), rng.gen());
                        }
                    },
                    2 => {
                        let data = unsafe {
                            slice::from_raw_parts_mut(
                                data.as_mut_ptr() as *mut u16,
                                data.len() / bytes_per_sample as usize,
                            )
                        };
                        for col in 0..data.len() {
                            unsafe {
                                ptr::write(data.as_mut_ptr().offset(col as isize), rng.gen());
                            }
                        }
                    }
                    4 => {
                        let data = unsafe {
                            slice::from_raw_parts_mut(
                                data.as_mut_ptr() as *mut u32,
                                data.len() / bytes_per_sample as usize,
                            )
                        };
                        for col in 0..data.len() {
                            unsafe {
                                ptr::write(data.as_mut_ptr().offset(col as isize), rng.gen());
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }

        Ok(Some(frame.into()))
    }

    fn get_frame(
        &self,
        _api: API,
        _core: CoreRef,
        _context: FrameContext,
        _n: usize,
    ) -> Result<FrameRef, Error> {
        unreachable!()
    }
}

export_vapoursynth_plugin! {
    Metadata {
        identifier: "com.example.vapoursynth-rs",
        namespace: "vapoursynth_rs",
        name: "Example vapoursynth-rs Plugin",
        read_only: true,
    },
    [Invert, RandomNoise]
}
