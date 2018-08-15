#![allow(unused)]
#[macro_use]
extern crate failure;
extern crate vapoursynth;

use failure::{err_msg, Error, ResultExt};
use std::env;
use vapoursynth::prelude::*;

fn usage() {
    println!(
        "Usage:\n\t{} <script.vpy> [frame number]",
        env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "vpy-info".to_owned())
    );
}

#[cfg(all(
    feature = "vsscript-functions",
    any(
        feature = "vapoursynth-functions",
        feature = "gte-vsscript-api-32"
    )
))]
fn print_node_info(node: &Node) {
    use std::fmt::Debug;

    // Helper function for printing properties.
    fn map_or_variable<T, F>(x: &Property<T>, f: F) -> String
    where
        T: Debug + Clone + Copy + Eq + PartialEq,
        F: FnOnce(&T) -> String,
    {
        match *x {
            Property::Variable => "variable".to_owned(),
            Property::Constant(ref x) => f(x),
        }
    }

    let info = node.info();

    println!(
        "Format: {}",
        map_or_variable(&info.format, |x| x.name().to_owned())
    );
    println!(
        "Resolution: {}",
        map_or_variable(&info.resolution, |x| format!("{}×{}", x.width, x.height))
    );
    println!(
        "Framerate: {}",
        map_or_variable(&info.framerate, |x| format!(
            "{}/{} ({})",
            x.numerator,
            x.denominator,
            x.numerator as f64 / x.denominator as f64
        ))
    );

    #[cfg(feature = "gte-vapoursynth-api-32")]
    println!("Frame count: {}", info.num_frames);

    #[cfg(not(feature = "gte-vapoursynth-api-32"))]
    println!(
        "Frame count: {}",
        map_or_variable(&info.num_frames, |x| format!("{}", x))
    );
}

#[cfg(all(
    feature = "vsscript-functions",
    any(
        feature = "vapoursynth-functions",
        feature = "gte-vsscript-api-32"
    )
))]
fn run() -> Result<(), Error> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| err_msg("The filename argument is missing"))?;
    let environment =
        vsscript::Environment::from_file(filename, vsscript::EvalFlags::SetWorkingDir)
            .context("Couldn't create the VSScript environment")?;

    let core = environment
        .get_core()
        .context("Couldn't get the VapourSynth core")?;
    println!("{}", core.info());

    #[cfg(feature = "gte-vsscript-api-31")]
    let (node, alpha_node) = environment
        .get_output(0)
        .context("Couldn't get the output at index 0")?;
    #[cfg(not(feature = "gte-vsscript-api-31"))]
    let (node, alpha_node) = (
        environment
            .get_output(0)
            .context("Couldn't get the output at index 0")?,
        None::<Node>,
    );

    print_node_info(&node);

    println!();
    if let Some(alpha_node) = alpha_node {
        println!("Alpha:");
        print_node_info(&alpha_node);
    } else {
        println!("Alpha: No");
    }

    if let Some(n) = env::args().nth(2) {
        let n = n
            .parse::<usize>()
            .context("Couldn't parse the frame number")?;
        if n > i32::max_value() as usize {
            bail!("Frame number is too big");
        }

        let frame = node.get_frame(n).context("Couldn't get the frame")?;

        println!();
        println!("Frame #{}", n);

        let format = frame.format();
        println!("Format: {}", format.name());
        println!("Plane count: {}", format.plane_count());

        let props = frame.props();
        let count = props.key_count();

        if count > 0 {
            println!();
        }

        for k in 0..count {
            let key = props.key(k);

            macro_rules! print_value {
                ($func:ident) => {
                    println!(
                        "Property: {} => {:?}",
                        key,
                        props.$func(key).unwrap().collect::<Vec<_>>()
                    )
                };
            }

            match props.value_type(key).unwrap() {
                ValueType::Int => print_value!(get_int_iter),
                ValueType::Float => print_value!(get_float_iter),
                ValueType::Data => print_value!(get_data_iter),
                ValueType::Node => print_value!(get_node_iter),
                ValueType::Frame => print_value!(get_frame_iter),
                ValueType::Function => print_value!(get_function_iter),
            }
        }

        for plane in 0..format.plane_count() {
            println!();
            println!("Plane #{}", plane);
            println!(
                "Resolution: {}×{}",
                frame.width(plane),
                frame.height(plane)
            );
            println!("Stride: {}", frame.stride(plane));
        }
    }

    Ok(())
}

#[cfg(not(all(
    feature = "vsscript-functions",
    any(
        feature = "vapoursynth-functions",
        feature = "gte-vsscript-api-32"
    )
)))]
fn run() -> Result<(), Error> {
    bail!(
        "This example requires the `vsscript-functions` and either `vapoursynth-functions` or \
         `vsscript-api-32` features."
    )
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err.cause());

        for cause in err.causes().skip(1) {
            eprintln!("Caused by: {}", cause);
        }

        eprintln!("{}", err.backtrace());

        usage();
    }
}
