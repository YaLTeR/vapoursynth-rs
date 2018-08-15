/// A test executable for the sample plugin.
#[macro_use]
extern crate cfg_if;
extern crate vapoursynth;
use vapoursynth::prelude::*;
use vapoursynth::video_info::Framerate;

use std::env::current_exe;
use std::fmt::Debug;
use std::io::{stdout, Write};

cfg_if! {
    if #[cfg(windows)] {
        const EXTENSION: &str = "dll";
        const PREFIX: &str = "";
    } else if #[cfg(target_os = "macos")] {
        const EXTENSION: &str = "dylib";
        const PREFIX: &str = "lib";
    } else {
        const EXTENSION: &str = "so";
        const PREFIX: &str = "lib";
    }
}

fn plugin_path() -> Vec<u8> {
    let mut path = current_exe().unwrap();
    path.set_file_name(format!("{}sample_plugin.{}", PREFIX, EXTENSION));
    path.into_os_string().into_string().unwrap().into_bytes()
}

fn make_environment() -> Environment {
    let env = Environment::new().unwrap();

    // Set the running_from_test variable.
    let api = API::get().unwrap();
    let mut map = OwnedMap::new(api);
    map.set("running_from_test", &1).unwrap();
    env.set_variables(&map).unwrap();

    // Load the required sample filter.
    {
        let core = env.get_core().unwrap();
        let std = core
            .get_plugin_by_id("com.vapoursynth.std")
            .unwrap()
            .unwrap();

        map.clear();
        map.set("path", &&plugin_path()[..]).unwrap();
        let rv = std.invoke("LoadPlugin", &map).unwrap();
        assert_eq!(rv.error(), None);
    }

    env
}

fn verify_pixels<T: Component + Copy + Debug + PartialEq>(frame: &Frame, expected: [T; 3]) {
    for plane_num in 0..3 {
        let expected_row = vec![expected[plane_num]; frame.width(plane_num)];

        for row in 0..frame.height(plane_num) {
            assert_eq!(&expected_row[..], frame.plane_row(plane_num, row));
        }
    }
}

fn test_passthrough() {
    print!("Running test_passthrough()...");
    stdout().flush().unwrap();

    let mut env = make_environment();
    env.eval_file("test-vpy/passthrough.vpy", EvalFlags::Nothing)
        .unwrap();
    let node = env.get_output(0).unwrap();

    verify_pixels::<u8>(&node.get_frame(0).unwrap(), [1 << 6, 1 << 6, 0]);
    verify_pixels::<u16>(&node.get_frame(1).unwrap(), [1 << 7, 1 << 7, 0]);
    verify_pixels::<u16>(&node.get_frame(2).unwrap(), [1 << 8, 1 << 8, 0]);
    verify_pixels::<u16>(&node.get_frame(3).unwrap(), [1 << 14, 1 << 14, 0]);

    println!(" ok");
}

fn test_invert() {
    print!("Running test_invert()...");
    stdout().flush().unwrap();

    let mut env = make_environment();
    env.eval_file("test-vpy/invert.vpy", EvalFlags::Nothing)
        .unwrap();
    let node = env.get_output(0).unwrap();

    verify_pixels::<u8>(
        &node.get_frame(0).unwrap(),
        [255 - (1 << 6), 255 - (1 << 6), 255],
    );
    verify_pixels::<u16>(
        &node.get_frame(1).unwrap(),
        [
            (1 << 9) - 1 - (1 << 7),
            (1 << 9) - 1 - (1 << 7),
            (1 << 9) - 1,
        ],
    );
    verify_pixels::<u16>(
        &node.get_frame(2).unwrap(),
        [
            (1 << 10) - 1 - (1 << 8),
            (1 << 10) - 1 - (1 << 8),
            (1 << 10) - 1,
        ],
    );
    verify_pixels::<u16>(
        &node.get_frame(3).unwrap(),
        [65535 - (1 << 14), 65535 - (1 << 14), 65535],
    );

    println!(" ok");
}

fn test_random_noise() {
    print!("Running test_random_noise()...");
    stdout().flush().unwrap();

    let mut env = make_environment();
    env.eval_file("test-vpy/random_noise.vpy", EvalFlags::Nothing)
        .unwrap();
    let node = env.get_output(0).unwrap();

    assert_eq!(node.info().num_frames, 10.into());
    assert_eq!(
        node.info().framerate,
        Framerate {
            numerator: 60,
            denominator: 1,
        }.into()
    );

    let frame = node.get_frame(0).unwrap();

    assert_eq!(frame.width(0), 320);
    assert_eq!(frame.height(0), 240);
    assert_eq!(frame.format().id(), PresetFormat::RGB24.into());

    println!(" ok");
}

fn test_make_random_noise() {
    print!("Running test_make_random_noise()...");
    stdout().flush().unwrap();

    let mut env = make_environment();
    env.eval_file("test-vpy/make_random_noise.vpy", EvalFlags::Nothing)
        .unwrap();
    let node = env.get_output(0).unwrap();

    assert_eq!(node.info().num_frames, 10.into());
    assert_eq!(
        node.info().framerate,
        Framerate {
            numerator: 60,
            denominator: 1,
        }.into()
    );

    let frame = node.get_frame(0).unwrap();

    assert_eq!(frame.width(0), 320);
    assert_eq!(frame.height(0), 240);
    assert_eq!(frame.format().id(), PresetFormat::RGB24.into());

    println!(" ok");
}

fn test_arguments() {
    print!("Running test_arguments()...");
    stdout().flush().unwrap();

    let mut env = make_environment();

    // If the evaluation succeeds, the test succeeds.
    env.eval_file("test-vpy/argument-test.vpy", EvalFlags::Nothing)
        .unwrap();

    println!(" ok");
}

fn main() {
    test_passthrough();
    test_invert();
    test_random_noise();
    test_make_random_noise();
    test_arguments();
}
