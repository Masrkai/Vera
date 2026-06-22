// build.rs
//
// In Python, the path to `main.slint` was resolved at runtime using `__file__`.
// In Rust, Slint files are compiled to native Rust code at build time via a
// build script. This script replicates the path resolution logic to find
// `ui/main.slint` relative to the project root.

use std::path::PathBuf;

fn main() {
    // CARGO_MANIFEST_DIR is the project root directory
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    // Construct the path to your .slint file
    let ui_file_path = manifest_dir.join("ui").join("main.slint");

    // OR if you moved just 'main.slint' directly to the root, use this instead:
    // let ui_file_path = manifest_dir.join("main.slint");

    // Compile the .slint file
    slint_build::compile(ui_file_path).expect("Slint build failed");
}
