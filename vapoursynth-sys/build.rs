use std::env;

fn main() {
    let windows = env::var("TARGET").unwrap().contains("windows");

    if let Ok(dir) = env::var("VAPOURSYNTH_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", dir);
    }

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
