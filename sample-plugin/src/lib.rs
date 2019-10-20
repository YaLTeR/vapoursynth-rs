///! A sample VapourSynth plugin.
#[macro_use]
extern crate failure;
extern crate rand;
#[macro_use]
extern crate vapoursynth;

use std::ffi::CStr;
use std::ptr;

use failure::{Error, ResultExt};
use rand::Rng;
use vapoursynth::core::CoreRef;
use vapoursynth::format::FormatID;
use vapoursynth::function::Function;
use vapoursynth::map::ValueIter;
use vapoursynth::node::Flags;
use vapoursynth::plugins::*;
use vapoursynth::prelude::*;
use vapoursynth::video_info::{Framerate, Resolution, VideoInfo};

const PLUGIN_IDENTIFIER: &str = "com.example.vapoursynth-rs";

// A simple filter that passes the frames through unchanged.
struct Passthrough<'core> {
    source: Node<'core>,
}

impl<'core> Filter<'core> for Passthrough<'core> {
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
        _core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<FrameRef<'core>, Error> {
        self.source
            .get_frame_filter(context, n)
            .ok_or_else(|| format_err!("Couldn't get the source frame"))
    }
}

make_filter_function! {
    PassthroughFunction, "Passthrough"

    fn create_passthrough<'core>(
        _api: API,
        _core: CoreRef<'core>,
        clip: Node<'core>,
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        Ok(Some(Box::new(Passthrough { source: clip })))
    }
}

// A filter that inverts the pixel values.
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
        let frame = self
            .source
            .get_frame_filter(context, n)
            .ok_or_else(|| format_err!("Couldn't get the source frame"))?;

        if frame.format().sample_type() == SampleType::Float {
            bail!("Floating point formats are not supported");
        }

        let mut frame = FrameRefMut::copy_of(core, &frame);

        for plane in 0..frame.format().plane_count() {
            for row in 0..frame.height(plane) {
                assert_eq!(frame.format().sample_type(), SampleType::Integer);

                let bits_per_sample = frame.format().bits_per_sample();
                let bytes_per_sample = frame.format().bytes_per_sample();

                match bytes_per_sample {
                    1 => for pixel in frame.plane_row_mut::<u8>(plane, row) {
                        *pixel = 255 - *pixel;
                    },
                    2 => for pixel in frame.plane_row_mut::<u16>(plane, row) {
                        *pixel = ((1u64 << bits_per_sample) - 1) as u16 - *pixel;
                    },
                    4 => for pixel in frame.plane_row_mut::<u32>(plane, row) {
                        *pixel = ((1u64 << bits_per_sample) - 1) as u32 - *pixel;
                    },
                    _ => unreachable!(),
                }
            }
        }

        Ok(frame.into())
    }
}

make_filter_function! {
    InvertFunction, "Invert"

    fn create_invert<'core>(
        _api: API,
        _core: CoreRef<'core>,
        clip: Node<'core>,
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        Ok(Some(Box::new(Invert { source: clip })))
    }
}

// A filter that outputs random noise.
struct RandomNoise {
    format_id: FormatID,
    resolution: Resolution,
    framerate: Framerate,
    length: usize,
}

impl<'core> Filter<'core> for RandomNoise {
    fn video_info(&self, _api: API, core: CoreRef<'core>) -> Vec<VideoInfo<'core>> {
        vec![VideoInfo {
            format: core.get_format(self.format_id).unwrap().into(),
            resolution: self.resolution.into(),
            framerate: self.framerate.into(),
            num_frames: self.length.into(),
            flags: Flags::empty(),
        }]
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

                match bytes_per_sample {
                    1 => {
                        let data = frame.plane_row_mut::<u8>(plane, row);
                        for col in 0..data.len() {
                            unsafe {
                                ptr::write(data.as_mut_ptr().add(col), rng.gen());
                            }
                        }
                    }
                    2 => {
                        let data = frame.plane_row_mut::<u16>(plane, row);
                        for col in 0..data.len() {
                            unsafe {
                                ptr::write(data.as_mut_ptr().add(col), rng.gen());
                            }
                        }
                    }
                    4 => {
                        let data = frame.plane_row_mut::<u32>(plane, row);
                        for col in 0..data.len() {
                            unsafe {
                                ptr::write(data.as_mut_ptr().add(col), rng.gen());
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

make_filter_function! {
    RandomNoiseFunction, "RandomNoise"

    fn create_random_noise<'core>(
        _api: API,
        core: CoreRef<'core>,
        format: i64,
        width: i64,
        height: i64,
        length: i64,
        fpsnum: i64,
        fpsden: i64,
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        let format_id = (format as i32).into();
        let format = core.get_format(format_id)
            .ok_or_else(|| format_err!("No such format"))?;

        if format.sample_type() == SampleType::Float {
            bail!("Floating point formats are not supported");
        }

        if width <= 0 || width > i64::from(i32::max_value()) {
            bail!("Invalid width");
        }
        let width = width as usize;

        if height <= 0 || height > i64::from(i32::max_value()) {
            bail!("Invalid height");
        }
        let height = height as usize;

        if length <= 0 || length > i64::from(i32::max_value()) {
            bail!("Invalid length");
        }
        let length = length as usize;

        if fpsnum <= 0 {
            bail!("Invalid fpsnum");
        }
        let fpsnum = fpsnum as u64;

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

// A random noise function but with variable name for MakeRandomNoiseFunction.
struct VariableNameRandomNoiseFunction {
    name: String,

    // So we don't have to implement args().
    underlying_function: RandomNoiseFunction,
}

impl FilterFunction for VariableNameRandomNoiseFunction {
    fn name(&self) -> &str {
        &self.name
    }

    fn args(&self) -> &str {
        self.underlying_function.args()
    }

    fn create<'core>(
        &self,
        api: API,
        core: CoreRef<'core>,
        args: &Map<'core>,
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        self.underlying_function.create(api, core, args)
    }
}

// A filter function that makes a random noise filter function with the given name at runtime.
make_filter_function! {
    MakeRandomNoiseFunction, "MakeRandomNoiseFilter"

    fn create_make_random_noise<'core>(
        _api: API,
        core: CoreRef<'core>,
        name: &[u8],
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        let name = unsafe { CStr::from_ptr(name.as_ptr() as _) };
        let name = name.to_str()
            .context("name contains invalid UTF-8")?
            .to_owned();

        let plugin = core.get_plugin_by_id(PLUGIN_IDENTIFIER).unwrap().unwrap();
        plugin
            .register_function(VariableNameRandomNoiseFunction {
                name,
                underlying_function: RandomNoiseFunction::new(),
            })
            .unwrap();

        Ok(None)
    }
}

// A filter for testing different kinds of argument passing.
struct ArgumentTestFilter<'core> {
    clip: Node<'core>,
}

impl<'core> Filter<'core> for ArgumentTestFilter<'core> {
    fn video_info(&self, _api: API, _core: CoreRef<'core>) -> Vec<VideoInfo<'core>> {
        vec![self.clip.info()]
    }

    fn get_frame_initial(
        &self,
        _api: API,
        _core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<Option<FrameRef<'core>>, Error> {
        self.clip.request_frame_filter(context, n);
        Ok(None)
    }

    fn get_frame(
        &self,
        _api: API,
        _core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<FrameRef<'core>, Error> {
        self.clip
            .get_frame_filter(context, n)
            .ok_or_else(|| format_err!("Couldn't get the source frame"))
    }
}

make_filter_function! {
    ArgumentTestFilterFunction, "ArgumentTest"

    fn create_argument_test<'core>(
        api: API,
        _core: CoreRef<'core>,
        int: i64,
        float: f64,
        data: &[u8],
        node: Node<'core>,
        frame: FrameRef<'core>,
        function: Function<'core>,
        optional_int: Option<i64>,
        another_optional_int: Option<i64>,
        frame_array: ValueIter<'_, 'core, FrameRef<'core>>,
        optional_frame_array: Option<ValueIter<'_, 'core, FrameRef<'core>>>,
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        let in_ = OwnedMap::new(api);
        let mut out = OwnedMap::new(api);
        function.call(&in_, &mut out);

        ensure!(int == 42, "{} != 42", int);
        #[allow(clippy::float_cmp)]
        {
            ensure!(float == 1337f64, "{} != 1337", float);
        }
        ensure!(data == &b"asd"[..], "{:?} != {:?}", data, &b"asd"[..]);
        ensure!(
            node.info().num_frames == Property::Constant(1),
            "{:?} != 1",
            node.info().num_frames
        );
        ensure!(frame.width(0) == 320, "{} != 320", frame.width(0));
        ensure!(out.get::<i64>("val").map(|x| x == 10).unwrap_or(false), "Incorrect function");
        ensure!(optional_int.is_some(), "optional_int is missing");
        ensure!(optional_int.unwrap() == 123, "{} != 123", optional_int.unwrap());
        ensure!(another_optional_int.is_none(), "another_optional_int was present");

        let mut frame_array = frame_array;
        ensure!(frame_array.len() == 2, "{} != 2", frame_array.len());
        let frame = frame_array.next().unwrap();
        ensure!(frame.width(0) == 256, "{} != 256", frame.width(0));
        let frame = frame_array.next().unwrap();
        ensure!(frame.width(0) == 64, "{} != 64", frame.width(0));

        ensure!(optional_frame_array.is_none(), "optional_frame_array was present");

        Ok(Some(Box::new(ArgumentTestFilter { clip: node })))
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
        PassthroughFunction::new(),
        InvertFunction::new(),
        RandomNoiseFunction::new(),
        MakeRandomNoiseFunction::new(),
        ArgumentTestFilterFunction::new(),
    ]
}
