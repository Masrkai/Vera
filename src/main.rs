// The include_modules!() macro exposes the generated types
// created by slint_build in build.rs
slint::include_modules!();

mod dialogs;
mod processing;
mod app_controller;

use app_controller::ExifBridge;

fn main() {
    // Initialize the bridge, which in turn instantiates the Slint UI component
    // and binds all the callbacks.
    let (bridge, ui) = ExifBridge::new();

    // Handle command line arguments: if an image file is provided, load it
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(path) = args.first() {
        // Check if the file exists
        if std::path::Path::new(path).exists() {
            // Load the image
            ExifBridge::load_image(&bridge, path.clone());
        } else {
            eprintln!("Warning: File '{}' does not exist", path);
        }
    }

    // Run the Slint event loop
    ExifBridge::run(&bridge, &ui);
}
