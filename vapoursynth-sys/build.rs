use std::env;

const LIBRARY_DIR_VARIABLE: &str = "VAPOURSYNTH_LIB_DIR";

fn main() {
    // Make sure the build script is re-run if our env variable is changed.
    println!("cargo:rerun-if-env-changed={}", LIBRARY_DIR_VARIABLE);

    let windows = env::var("TARGET").unwrap().contains("windows");

    // Library directory override.
    if let Ok(dir) = env::var(LIBRARY_DIR_VARIABLE) {
        println!("cargo:rustc-link-search=native={}", dir);
    }

    // Handle linking to VapourSynth libs.
    if env::var("CARGO_FEATURE_VAPOURSYNTH_FUNCTIONS").is_ok() {
        println!("cargo:rustc-link-lib=vapoursynth");
    }

    if env::var("CARGO_FEATURE_VSSCRIPT_FUNCTIONS").is_ok() {
        let vsscript_lib_name = if windows {
            "vsscript"
        } else {
            "vapoursynth-script"
        };

        println!("cargo:rustc-link-lib={}", vsscript_lib_name);
    }
}
