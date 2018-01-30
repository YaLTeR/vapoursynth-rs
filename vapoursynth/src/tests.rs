#![cfg(test)]

use super::*;
use video_info::{Framerate, Resolution};

#[cfg(all(feature = "vapoursynth-functions", feature = "vsscript-functions"))]
#[test]
fn green() {
    let api = API::get().unwrap();
    let env = vsscript::Environment::from_file("test-vpy/green.vpy", vsscript::EvalFlags::Nothing)
        .unwrap();
    let node = env.get_output(api, 0).unwrap();
    let info = node.info();

    if let Property::Constant(format) = info.format {
        assert_eq!(format.name().to_string_lossy(), "RGB24");
    } else {
        assert!(false);
    }

    assert_eq!(
        info.framerate,
        Property::Constant(Framerate {
            numerator: 60,
            denominator: 1,
        })
    );
    assert_eq!(
        info.resolution,
        Property::Constant(Resolution {
            width: 1920,
            height: 1080,
        })
    );

    #[cfg(feature = "gte-vapoursynth-api-32")]
    assert_eq!(info.num_frames, 100);
    #[cfg(not(feature = "gte-vapoursynth-api-32"))]
    assert_eq!(info.num_frames, Property::Constant(100));

    let frame = node.get_frame(0).unwrap();
    let format = frame.format();
    assert_eq!(format.name().to_string_lossy(), "RGB24");
    assert_eq!(format.plane_count(), 3);

    for plane in 0..format.plane_count() {
        let resolution = frame.resolution(plane);
        assert_eq!(
            resolution,
            Resolution {
                width: 1920,
                height: 1080,
            }
        );

        let color = if plane == 1 { [255; 1920] } else { [0; 1920] };

        let stride = frame.stride(plane);
        let plane = frame.data(plane);

        for row in 0..resolution.height {
            assert_eq!(
                &plane[row * stride..row * stride + resolution.width],
                &color[..]
            );
        }
    }
}

#[cfg(all(feature = "vapoursynth-functions", feature = "vsscript-functions"))]
#[test]
fn green_from_string() {
    let api = API::get().unwrap();
    let env = vsscript::Environment::from_script(include_str!("../test-vpy/green.vpy")).unwrap();
    let node = env.get_output(api, 0).unwrap();
    let info = node.info();

    if let Property::Constant(format) = info.format {
        assert_eq!(format.name().to_string_lossy(), "RGB24");
    } else {
        assert!(false);
    }

    assert_eq!(
        info.framerate,
        Property::Constant(Framerate {
            numerator: 60,
            denominator: 1,
        })
    );
    assert_eq!(
        info.resolution,
        Property::Constant(Resolution {
            width: 1920,
            height: 1080,
        })
    );

    #[cfg(feature = "gte-vapoursynth-api-32")]
    assert_eq!(info.num_frames, 100);
    #[cfg(not(feature = "gte-vapoursynth-api-32"))]
    assert_eq!(info.num_frames, Property::Constant(100));

    let frame = node.get_frame(0).unwrap();
    let format = frame.format();
    assert_eq!(format.name().to_string_lossy(), "RGB24");
    assert_eq!(format.plane_count(), 3);

    for plane in 0..format.plane_count() {
        let resolution = frame.resolution(plane);
        assert_eq!(
            resolution,
            Resolution {
                width: 1920,
                height: 1080,
            }
        );

        let color = if plane == 1 { [255; 1920] } else { [0; 1920] };

        let stride = frame.stride(plane);
        let plane = frame.data(plane);

        for row in 0..resolution.height {
            assert_eq!(
                &plane[row * stride..row * stride + resolution.width],
                &color[..]
            );
        }
    }
}

#[cfg(all(feature = "vapoursynth-functions", feature = "vsscript-functions"))]
#[test]
fn variable() {
    let api = API::get().unwrap();
    let env =
        vsscript::Environment::from_file("test-vpy/variable.vpy", vsscript::EvalFlags::Nothing)
            .unwrap();
    let node = env.get_output(api, 0).unwrap();
    let info = node.info();

    assert_eq!(info.format, Property::Variable);
    assert_eq!(info.framerate, Property::Variable);
    assert_eq!(info.resolution, Property::Variable);

    #[cfg(feature = "gte-vapoursynth-api-32")]
    assert_eq!(info.num_frames, 200);
    #[cfg(not(feature = "gte-vapoursynth-api-32"))]
    assert_eq!(info.num_frames, Property::Constant(200));

    // Test the first frame.
    let frame = node.get_frame(0).unwrap();
    let format = frame.format();
    assert_eq!(format.name().to_string_lossy(), "RGB24");
    assert_eq!(format.plane_count(), 3);

    for plane in 0..format.plane_count() {
        let resolution = frame.resolution(plane);
        assert_eq!(
            resolution,
            Resolution {
                width: 1920,
                height: 1080,
            }
        );

        let color = if plane == 1 { [255; 1920] } else { [0; 1920] };

        let stride = frame.stride(plane);
        let plane = frame.data(plane);

        for row in 0..resolution.height {
            assert_eq!(&plane[row * stride..(row + 1) * stride], &color[..]);
        }
    }

    // Test the first frame of the next format.
    let frame = node.get_frame(100).unwrap();
    let format = frame.format();
    assert_eq!(format.name().to_string_lossy(), "Gray8");
    assert_eq!(format.plane_count(), 1);

    let plane = 0;
    let resolution = frame.resolution(plane);
    assert_eq!(
        resolution,
        Resolution {
            width: 1280,
            height: 720,
        }
    );

    let color = [127; 1280];

    let stride = frame.stride(plane);
    let plane = frame.data(plane);

    for row in 0..resolution.height {
        assert_eq!(
            &plane[row * stride..row * stride + resolution.width],
            &color[..]
        );
    }
}

#[cfg(all(feature = "vapoursynth-functions", feature = "vsscript-functions"))]
#[test]
fn clear_output() {
    let env = vsscript::Environment::from_script(include_str!("../test-vpy/green.vpy")).unwrap();
    assert!(env.clear_output(1).is_none());
    assert!(env.clear_output(0).is_some());
    assert!(env.clear_output(0).is_none());
}
