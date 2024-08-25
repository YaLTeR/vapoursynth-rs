use std::env;
use std::path::PathBuf;

const LIBRARY_DIR_VARIABLE: &str = "VAPOURSYNTH_LIB_DIR";

fn main() {
    // Make sure the build script is re-run if our env variable is changed.
    println!("cargo:rerun-if-env-changed={}", LIBRARY_DIR_VARIABLE);

    // These should always be set when a build script is run
    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();

    let targets_windows = target.contains("windows");
    let targets_macos = target.contains("apple-darwin");

    // Get the default library dir for some platforms.
    let default_library_dir = if targets_windows {
        get_default_windows_library_dir(target, host)
    } else if targets_macos {
        get_default_macos_library_dir(target, host)
    } else {
        vec![]
    };

    // Library directory override or the default dir on windows.
    if let Ok(dir) = env::var(LIBRARY_DIR_VARIABLE) {
        println!("cargo:rustc-link-search=native={}", dir);
    } else {
        for dir in default_library_dir {
            println!("cargo:rustc-link-search=native={}", dir);
        }
    }

    // Handle linking to VapourSynth libs.
    if env::var("CARGO_FEATURE_VAPOURSYNTH_FUNCTIONS").is_ok() {
        println!("cargo:rustc-link-lib=vapoursynth");
    }

    if env::var("CARGO_FEATURE_VSSCRIPT_FUNCTIONS").is_ok() {
        let vsscript_lib_name = if targets_windows {
            "vsscript"
        } else {
            "vapoursynth-script"
        };

        println!("cargo:rustc-link-lib={}", vsscript_lib_name);
    }
}

// Returns the default library dirs on Windows.
// The default dir is where the VapourSynth installer puts the libraries.
fn get_default_windows_library_dir(target: String, host: String) -> Vec<String> {
    // If the host isn't Windows we don't have %programfiles%.
    if !host.contains("windows") {
        return vec![];
    }

    let programfiles = env::var("programfiles").into_iter();

    // Add Program Files from the other bitness. This would be Program Files (x86) with a 64-bit
    // host and regular Program Files with a 32-bit host running on a 64-bit system.
    let programfiles = programfiles.chain(env::var(if host.starts_with("i686") {
        "programw6432"
    } else {
        "programfiles(x86)"
    }));

    let suffix = if target.starts_with("i686") {
        "lib32"
    } else {
        "lib64"
    };

    programfiles
        .flat_map(move |programfiles| {
            // Use both VapourSynth and VapourSynth-32 folder names.
            ["", "-32"].iter().filter_map(move |vapoursynth_suffix| {
                let mut path = PathBuf::from(&programfiles);
                path.push(format!("VapourSynth{}", vapoursynth_suffix));
                path.push("sdk");
                path.push(suffix);
                path.to_str().map(|s| s.to_owned())
            })
        })
        .collect()
}

// Returns the default library dirs on macOS when using homebrew.
fn get_default_macos_library_dir(target: String, host: String) -> Vec<String> {
    // If the host is not macOS/Apple, the library dirs will be different.
    if !host.contains("apple-darwin") {
        return vec![];
    }

    if target.starts_with("aarch64") {
        vec![String::from("/opt/homebrew/lib/")]
    } else {
        vec![String::from("/usr/local/homebrew/lib/")]
    }
}
