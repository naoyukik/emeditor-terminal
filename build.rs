use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is not set; this build script must be run by Cargo with CARGO_MANIFEST_DIR defined");
    let manifest_path = PathBuf::from(manifest_dir);
    let def_file = manifest_path.join("exports.def");

    // Link with the .def file to ensure exported function names are not mangled
    println!("cargo:rustc-cdylib-link-arg=/DEF:{}", def_file.display());

    // Generate resource.rs from resource.h
    generate_resource_rs(&manifest_path);

    // Compile resource file
    embed_resource::compile("emeditor-terminal.rc", embed_resource::NONE);
}

fn generate_resource_rs(manifest_path: &std::path::Path) {
    let header_path = manifest_path.join("resource.h");
    let output_path = manifest_path.join("src/gui/driver/resource.rs");

    if let Ok(content) = fs::read_to_string(&header_path) {
        let mut rust_code = String::from("//! Generated from resource.h by build.rs\n\n");
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[0] == "#define" {
                let name = parts[1];
                let value = parts[2];
                // Generate all as i32 to support -1 (IDC_STATIC) and avoid type mismatches
                if (name.starts_with("ID") || name.starts_with("IDC_"))
                    && (value.chars().all(|c| c.is_numeric() || c == '-')
                        || value.starts_with("0x"))
                {
                    rust_code.push_str(&format!("pub const {}: i32 = {};\n", name, value));
                }
            }
        }
        fs::write(output_path, rust_code).expect("Failed to write resource.rs");
    }
}
