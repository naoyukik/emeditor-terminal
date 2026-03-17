use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is not set; this build script must be run by Cargo with CARGO_MANIFEST_DIR defined");
    let manifest_path = PathBuf::from(manifest_dir);
    let def_file = manifest_path.join("exports.def");

    // Tell Cargo to rerun this script if these files change
    println!("cargo:rerun-if-changed=resource.h");
    println!("cargo:rerun-if-changed=emeditor-terminal.rc");
    println!("cargo:rerun-if-changed=exports.def");
    println!("cargo:rerun-if-changed=build.rs");

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

    // Ensure resource.h exists and read it
    let content = fs::read_to_string(&header_path)
        .unwrap_or_else(|_| panic!("Failed to read resource.h at {}", header_path.display()));

    let mut rust_code = String::from("//! Generated from resource.h by build.rs\n\n");
    for line in content.lines() {
        let line = line.trim();
        if !line.starts_with("#define") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let name = parts[1];
        let value_raw = parts[2];

        // Only process resource IDs (ID* or IDC_*)
        if !(name.starts_with("ID") || name.starts_with("IDC_")) {
            continue;
        }

        // Robust parsing: remove comments and parentheses
        let value_no_comment = value_raw.split("//").next().unwrap().trim();
        let value_clean = value_no_comment.trim_matches(|c| c == '(' || c == ')');

        // Check if the value is a valid numeric (decimal or hex)
        let is_hex = value_clean.starts_with("0x");
        let is_valid = if is_hex {
            value_clean[2..].chars().all(|c| c.is_ascii_hexdigit())
        } else {
            value_clean.chars().all(|c| c.is_numeric() || c == '-')
        };

        if is_valid {
            rust_code.push_str(&format!(
                "#[allow(dead_code)]\npub const {}: i32 = {};\n",
                name, value_clean
            ));
        }
    }

    // Only write if content has changed to avoid unnecessary rebuilds
    let should_write = match fs::read_to_string(&output_path) {
        Ok(existing) => existing != rust_code,
        Err(_) => true,
    };

    if should_write {
        fs::write(&output_path, rust_code)
            .unwrap_or_else(|_| panic!("Failed to write resource.rs at {}", output_path.display()));
    }
}
