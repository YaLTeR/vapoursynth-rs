extern crate failure;
extern crate vapoursynth;

use failure::{err_msg, Error, ResultExt};
use std::borrow::Cow;
use std::env;

fn usage() {
    println!(
        "Usage:\n\t{} <script.vpy>",
        env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "vpy-info".to_owned())
    );
}

#[cfg(all(feature = "vapoursynth-functions", feature = "vsscript-functions"))]
fn run() -> Result<(), Error> {
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

    println!("{:#?}", node.info());

    let format_name = match node.info().format {
        Property::Variable => Cow::Borrowed("variable"),
        Property::Constant(f) => f.name().to_string_lossy(),
    };
    println!("Format name: {}", format_name);

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
