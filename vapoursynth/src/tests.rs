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
}
