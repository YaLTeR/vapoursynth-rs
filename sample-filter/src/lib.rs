///! A sample VapourSynth plugin.
#[macro_use]
extern crate failure;
extern crate rand;
#[macro_use]
extern crate vapoursynth;

use std::ffi::CStr;
use std::{ptr, slice};

use failure::{Error, ResultExt};
use rand::Rng;
use vapoursynth::prelude::*;
use vapoursynth::plugins::*;
use vapoursynth::core::CoreRef;
use vapoursynth::format::FormatID;
use vapoursynth::node::Flags;
use vapoursynth::video_info::{Framerate, Resolution, VideoInfo};

const PLUGIN_IDENTIFIER: &str = "com.example.vapoursynth-rs";

// A filter that inverts the pixel values.
struct InvertFunction;

impl FilterFunction for InvertFunction {
    fn name(&self) -> &str {
        "Invert"
    }

    fn args(&self) -> &str {
        "clip:clip"
    }

    fn create<'core>(
        &self,
        _api: API,
        _core: CoreRef<'core>,
        args: &Map<'core>,
    ) -> Result<Option<Box<Filter<'core> + 'core>>, Error> {
        let source = args.get_node("clip").unwrap();
        Ok(Some(Box::new(Invert { source })))
    }
}

struct Invert<'core> {
    source: Node<'core>,
}

impl<'core> Filter<'core> for Invert<'core> {
    fn video_info(&self, _api: API, _core: CoreRef<'core>) -> Vec<VideoInfo<'core>> {
        vec![self.source.info()]
    }

    fn get_frame_initial(
        &self,
        _api: API,
        _core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<Option<FrameRef<'core>>, Error> {
        self.source.request_frame_filter(context, n);
        Ok(None)
    }

    fn get_frame(
        &self,
        _api: API,
        core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<FrameRef<'core>, Error> {
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

// A filter that outputs random noise.
struct RandomNoiseFunction {
    // Store the name for the MakeRandomNoiseFunction example.
    name: String,
}

impl FilterFunction for RandomNoiseFunction {
    fn name(&self) -> &str {
        &self.name
    }

    fn args(&self) -> &str {
        "width:int;height:int;format:int;length:int;fpsnum:int;fpsden:int"
    }

    fn create<'core>(
        &self,
        _api: API,
        core: CoreRef<'core>,
        args: &Map<'core>,
    ) -> Result<Option<Box<Filter<'core> + 'core>>, Error> {
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

        Ok(Some(Box::new(RandomNoise {
            format_id,
            resolution: Resolution { width, height },
            framerate: Framerate {
                numerator: fpsnum,
                denominator: fpsden,
            },
            length,
        })))
    }
}

struct RandomNoise {
    format_id: FormatID,
    resolution: Resolution,
    framerate: Framerate,
    length: usize,
}

impl<'core> Filter<'core> for RandomNoise {
    fn video_info(&self, _api: API, core: CoreRef<'core>) -> Vec<VideoInfo<'core>> {
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
        core: CoreRef<'core>,
        _context: FrameContext,
        _n: usize,
    ) -> Result<Option<FrameRef<'core>>, Error> {
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
        _core: CoreRef<'core>,
        _context: FrameContext,
        _n: usize,
    ) -> Result<FrameRef<'core>, Error> {
        unreachable!()
    }
}

// A filter function that makes a random noise filter function with the given name at runtime.
struct MakeRandomNoiseFunction;

impl FilterFunction for MakeRandomNoiseFunction {
    fn name(&self) -> &str {
        "MakeRandomNoiseFilter"
    }

    fn args(&self) -> &str {
        "name:data"
    }

    fn create<'core>(
        &self,
        _api: API,
        core: CoreRef<'core>,
        args: &Map<'core>,
    ) -> Result<Option<Box<Filter<'core> + 'core>>, Error> {
        let name = unsafe { CStr::from_ptr(args.get_data("name").unwrap().as_ptr() as _) };
        let name = name.to_str()
            .context("name contains invalid UTF-8")?
            .to_owned();

        let plugin = core.get_plugin_by_id(PLUGIN_IDENTIFIER).unwrap().unwrap();
        plugin
            .register_function(RandomNoiseFunction { name })
            .unwrap();

        Ok(None)
    }
}

export_vapoursynth_plugin! {
    Metadata {
        identifier: PLUGIN_IDENTIFIER,
        namespace: "vapoursynth_rs",
        name: "Example vapoursynth-rs Plugin",
        read_only: false,
    },
    [
        InvertFunction,
        RandomNoiseFunction { name: "RandomNoise".to_owned() },
        MakeRandomNoiseFunction,
    ]
}
