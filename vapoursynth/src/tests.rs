#![cfg(test)]
use super::*;

// We need the VSScript functions, and either VSScript API 3.2 or the VapourSynth functions.
#[cfg(all(feature = "vsscript-functions",
          any(feature = "vapoursynth-functions", feature = "gte-vsscript-api-32")))]
mod need_api_and_vsscript {
    use std::sync::mpsc::channel;

    use super::*;
    use video_info::{Framerate, Resolution};

    fn props_test(frame: &Frame, fps_num: i64) {
        let props = frame.props();
        assert_eq!(props.key_count(), 2);
        assert_eq!(props.key(0), "_DurationDen");
        assert_eq!(props.key(1), "_DurationNum");

        assert_eq!(props.value_count(props.key(0)), Ok(1));
        if let Ok(Value::Int(x)) = props.value(props.key(0), 0) {
            assert_eq!(x, fps_num);
        } else {
            assert!(false);
        }
        assert_eq!(props.value_count(props.key(1)), Ok(1));
        if let Ok(Value::Int(1)) = props.value(props.key(1), 0) {
        } else {
            assert!(false);
        }
    }

    fn env_video_var_test(api: API, env: &vsscript::Environment) {
        let mut map = Map::new(api);
        assert!(env.get_variable("video", &mut map.get_ref_mut()).is_ok());
        let value = map.iter().next();
        assert!(value.is_some());
        let value = value.unwrap();
        assert_eq!(value.0, "video");
        if let ValueArray::Nodes(x) = value.1 {
            assert_eq!(x.len(), 1);
        } else {
            assert!(false);
        }
    }

    fn green_frame_test(frame: &Frame) {
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

            for row in 0..resolution.height {
                let data_row = frame.data_row(plane, row);
                assert_eq!(&data_row[..], &color[..]);
            }
        }
    }

    fn green_test(env: &vsscript::Environment) {
        let api = API::get().unwrap();
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
        green_frame_test(&frame);
        props_test(&frame, 60);
        env_video_var_test(api, &env);
    }

    #[test]
    fn green() {
        let env =
            vsscript::Environment::from_file("test-vpy/green.vpy", vsscript::EvalFlags::Nothing)
                .unwrap();
        green_test(&env);
    }

    #[test]
    fn green_from_string() {
        let env =
            vsscript::Environment::from_script(include_str!("../test-vpy/green.vpy")).unwrap();
        green_test(&env);
    }

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

            for row in 0..resolution.height {
                let data_row = frame.data_row(plane, row);
                assert_eq!(&data_row[..], &color[..]);
            }
        }

        props_test(&frame, 60);

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

        for row in 0..resolution.height {
            let data_row = frame.data_row(plane, row);
            assert_eq!(&data_row[..], &color[..]);
        }

        props_test(&frame, 30);
        env_video_var_test(api, &env);
    }

    #[test]
    fn clear_output() {
        let env =
            vsscript::Environment::from_script(include_str!("../test-vpy/green.vpy")).unwrap();
        assert!(env.clear_output(1).is_none());
        assert!(env.clear_output(0).is_some());
        assert!(env.clear_output(0).is_none());
    }

    #[test]
    fn iterators() {
        let api = API::get().unwrap();
        let env =
            vsscript::Environment::from_script(include_str!("../test-vpy/green.vpy")).unwrap();
        let node = env.get_output(api, 0).unwrap();
        let frame = node.get_frame(0).unwrap();
        let props = frame.props();

        assert_eq!(props.keys().size_hint(), (2, Some(2)));
        assert_eq!(props.iter().size_hint(), (2, Some(2)));
    }

    #[test]
    fn vsscript_variables() {
        let api = API::get().unwrap();
        let env =
            vsscript::Environment::from_script(include_str!("../test-vpy/green.vpy")).unwrap();

        let mut map = Map::new(api);
        assert!(env.get_variable("video", &mut map.get_ref_mut()).is_ok());
        assert!(env.clear_variable("video").is_ok());
        assert!(env.clear_variable("video").is_err());
        assert!(env.get_variable("video", &mut map.get_ref_mut()).is_err());

        assert!(env.set_variables(&map.get_ref()).is_ok());
        assert!(env.get_variable("video", &mut map.get_ref_mut()).is_ok());
    }

    #[test]
    fn get_frame_async() {
        let api = API::get().unwrap();
        let env =
            vsscript::Environment::from_file("test-vpy/green.vpy", vsscript::EvalFlags::Nothing)
                .unwrap();
        let node = env.get_output(api, 0).unwrap();

        let mut rxs = Vec::new();

        for i in 0..10 {
            let (tx, rx) = channel();
            rxs.push(rx);

            node.get_frame_async(i, move |frame, n, node| {
                assert!(frame.is_ok());
                let frame = frame.unwrap();

                assert_eq!(n, i);
                assert_eq!(
                    node.info().framerate,
                    Property::Constant(Framerate {
                        numerator: 60,
                        denominator: 1,
                    })
                );

                green_frame_test(&frame);
                props_test(&frame, 60);

                assert_eq!(tx.send(()), Ok(()));
            });
        }

        drop(node); // Test dropping prematurely.

        for rx in rxs {
            assert_eq!(rx.recv(), Ok(()));
        }
    }

    #[test]
    fn get_frame_async_error() {
        let api = API::get().unwrap();
        let env =
            vsscript::Environment::from_file("test-vpy/green.vpy", vsscript::EvalFlags::Nothing)
                .unwrap();
        let node = env.get_output(api, 0).unwrap();

        let (tx, rx) = channel();

        // The clip only has 100 frames, so requesting the 101th one produces an error.
        node.get_frame_async(100, move |frame, n, node| {
            assert!(frame.is_err());
            assert_eq!(n, 100);
            assert_eq!(
                node.info().framerate,
                Property::Constant(Framerate {
                    numerator: 60,
                    denominator: 1,
                })
            );

            assert_eq!(tx.send(()), Ok(()));
        });

        assert_eq!(rx.recv(), Ok(()));
    }

    #[test]
    fn core() {
        let api = API::get().unwrap();
        let env =
            vsscript::Environment::from_file("test-vpy/green.vpy", vsscript::EvalFlags::Nothing)
                .unwrap();

        let core = env.get_core(api);
        assert!(core.is_some());
    }
}

// We need either VSScript API 3.2 or the VapourSynth functions.
#[cfg(any(feature = "vapoursynth-functions",
          all(feature = "vsscript-functions", feature = "gte-vsscript-api-32")))]
mod need_api {
    use std::ffi::CString;
    use std::sync::mpsc::channel;

    use super::*;

    #[test]
    fn maps() {
        let api = API::get().unwrap();
        let mut map = Map::new(api);

        assert_eq!(map.key_count(), 0);

        assert_eq!(map.touch("test_frame", ValueType::Frame), Ok(()));
        assert_eq!(map.value_type("test_frame"), Ok(ValueType::Frame));
        assert_eq!(map.value_count("test_frame"), Ok(0));

        assert_eq!(
            map.append_value("test_frame", ValueRef::Int(0)),
            Err(Error::WrongValueType)
        );
        assert_eq!(map.set_value("test_frame", ValueRef::Float(1f64)), Ok(()));
        if let Ok(Value::Float(x)) = map.value("test_frame", 0) {
            assert_eq!(x, 1f64);
        } else {
            assert!(false);
        }

        const TEST_DATA: &[&[u8]] = &[&[0, 1, 2], &[3, 4, 5], &[1], &[3]];
        assert_eq!(
            map.set_values("test_frame", Values::Data(&mut TEST_DATA.iter().cloned())),
            Ok(())
        );
        if let Ok(ValueArray::Data(xs)) = map.values("test_frame") {
            assert_eq!(xs, TEST_DATA);
        } else {
            assert!(false);
        }

        assert_eq!(
            map.set_values("other", Values::IntArray(&[1, 2, 3])),
            Ok(())
        );
        if let Ok(ValueArray::Ints(xs)) = map.values("other") {
            assert_eq!(xs, &[1, 2, 3]);
        } else {
            assert!(false);
        }

        let clone = map.clone();
        assert_eq!(
            map.keys().collect::<Vec<_>>(),
            clone.keys().collect::<Vec<_>>()
        );
        if let Ok(ValueArray::Data(xs)) = clone.values("test_frame") {
            assert_eq!(xs, TEST_DATA);
        } else {
            assert!(false);
        }

        if let Ok(ValueArray::Ints(xs)) = clone.values("other") {
            assert_eq!(xs, &[1, 2, 3]);
        } else {
            assert!(false);
        }

        assert_eq!(map.delete_key("test_frame"), Ok(()));
        assert_eq!(map.delete_key("test_frame"), Err(Error::KeyNotFound));

        if let Ok(ValueIterEnum::Data(mut iter)) = clone.value_iter("test_frame") {
            assert_eq!(iter.next(), Some(TEST_DATA[0]));
            assert_eq!(iter.next(), Some(TEST_DATA[1]));
            assert_eq!(iter.next(), Some(TEST_DATA[2]));
            assert_eq!(iter.next(), Some(TEST_DATA[3]));
            assert_eq!(iter.next(), None);
        } else {
            assert!(false);
        }

        assert_eq!(map.error(), None);
        assert_eq!(map.set_error("hello there"), Ok(()));
        assert_eq!(
            map.error().as_ref().map(|x| x.as_ref()),
            Some("hello there")
        );
    }

    #[cfg(feature = "gte-vapoursynth-api-34")]
    #[test]
    fn message_handler() {
        let api = API::get().unwrap();
        let (tx, rx) = channel();

        // Hopefully no one logs anything here and breaks the test.
        api.set_message_handler(move |message_type, message| {
            assert_eq!(tx.send((message_type, message.to_owned())), Ok(()));
        });

        assert_eq!(
            api.log(MessageType::Warning, "test warning message"),
            Ok(())
        );
        assert_eq!(
            rx.recv(),
            Ok((
                MessageType::Warning,
                CString::new("test warning message").unwrap()
            ))
        );

        assert_eq!(api.log(MessageType::Debug, "test debug message"), Ok(()));
        assert_eq!(
            rx.recv(),
            Ok((
                MessageType::Debug,
                CString::new("test debug message").unwrap()
            ))
        );

        assert_eq!(
            api.log(MessageType::Critical, "test critical message"),
            Ok(())
        );
        assert_eq!(
            rx.recv(),
            Ok((
                MessageType::Critical,
                CString::new("test critical message").unwrap()
            ))
        );

        api.clear_message_handler();
    }
}
