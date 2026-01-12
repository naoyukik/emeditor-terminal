fn main() {
    // Link with the .def file to ensure exported function names are not mangled
    println!("cargo:rustc-cdylib-link-arg=/DEF:exports.def");
}
