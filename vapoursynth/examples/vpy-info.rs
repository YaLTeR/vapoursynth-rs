extern crate failure;
extern crate vapoursynth;

use failure::{err_msg, Error, ResultExt};
use std::env;

fn usage() {
    println!(
        "Usage:\n\t{} <script.vpy> [frame number]",
        env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "vpy-info".to_owned())
    );
}

#[cfg(all(feature = "vapoursynth-functions", feature = "vsscript-functions"))]
fn run() -> Result<(), Error> {
    use std::fmt::Debug;
    use vapoursynth::{vsscript, Property};

    let filename = env::args()
        .nth(1)
        .ok_or_else(|| err_msg("The filename argument is missing"))?;
    let api = vapoursynth::API::get().ok_or_else(|| err_msg("Couldn't get the VapourSynth API"))?;
    let environment =
        vsscript::Environment::from_file(filename, vsscript::EvalFlags::SetWorkingDir)
            .context("Couldn't create the VSScript environment")?;
    let node = environment
        .get_output(api, 0)
        .ok_or_else(|| err_msg("No output at index 0"))?;
    let info = node.info();

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

    println!(
        "Format: {}",
        map_or_variable(&info.format, |x| x.name().to_string_lossy().into_owned())
    );
    println!(
        "Resolution: {}",
        map_or_variable(&info.resolution, |x| format!("{}×{}", x.width, x.height))
    );
    println!(
        "Framerate: {}",
        map_or_variable(&info.framerate, |x| {
            format!(
                "{}/{} ({})",
                x.numerator,
                x.denominator,
                x.numerator as f64 / x.denominator as f64
            )
        })
    );

    #[cfg(feature = "gte-vapoursynth-api-32")]
    println!("Frame count: {}", info.num_frames);

    #[cfg(not(feature = "gte-vapoursynth-api-32"))]
    println!(
        "Frame count: {}",
        map_or_variable(&info.num_frames, |x| format!("{}", x))
    );

    if let Some(n) = env::args().nth(2) {
        let n = n.parse::<usize>()
            .context("Couldn't parse the frame number")?;
        if n > i32::max_value() as usize {
            return Err(err_msg("Frame number is too big"));
        }

        let frame = node.get_frame(n)
            .map_err(|e| err_msg(e.to_string_lossy().into_owned()))
            .context("Couldn't get the frame")?;

        println!("");
        println!("Frame #{}", n);

        let format = frame.format();
        println!("Format: {}", format.name().to_string_lossy());
        println!("Plane count: {}", format.plane_count());

        for plane in 0..format.plane_count() {
            println!("");
            println!("Plane #{}", plane);
            println!("Resolution: {}×{}", frame.width(plane), frame.height(plane));
            println!("Stride: {}", frame.stride(plane));
        }
    }

    Ok(())
}

#[cfg(not(all(feature = "vapoursynth-functions", feature = "vsscript-functions")))]
fn run() -> Result<(), Error> {
    Err(err_msg(
        "This example requires the `vapoursynth-functions vsscript-functions` features.",
    ))
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
