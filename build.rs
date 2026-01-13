use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is not set; this build script must be run by Cargo with CARGO_MANIFEST_DIR defined");
    let def_file = PathBuf::from(manifest_dir).join("exports.def");

    // Link with the .def file to ensure exported function names are not mangled
    println!("cargo:rustc-cdylib-link-arg=/DEF:{}", def_file.display());
}
