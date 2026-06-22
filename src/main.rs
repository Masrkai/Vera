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

    // Run the Slint event loop
    ExifBridge::run(&bridge, &ui);
}
