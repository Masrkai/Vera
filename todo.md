# TODO List

- [x] The basic main goal of making a drop-in Google Maps viewer using image metadata, while simultaneously being able to see, edit, or delete metadata in single or multiple batches.

## Bugs to Fix

### Logic

- [x] Clicking "open image" while another image is already open seems to increase memory usage drastically.

### Bundle & Build

- [x] Using the GUI without the `SLINT_BACKEND=winit-software cargo run` flag makes the GUI render using OpenGL, which consumes much more memory.
- [ ] The Slint package in Cargo is extremely dense; building requires compiling around `674` packages using `cargo build --release`.
- [ ] Ship the application properly on Linux using freedesktop.org standards (app icon and `.desktop` file).
  - [x] NixOS
  - [ ] Standard Linux Desktop

## Future List

- [ ] Add an option to view locations using ESRI satellite imagery.
- [ ] Add an XDG standard for opening image files through the options called "open with" then by selecting "vera" it opens the image directly!.
- [ ] Add logic for more forensic analysis. (this will be the most huge ask or a separate project entirely)
  - [ ] Add patterns for Encrypting and Decrypting a message from an image.
    - [ ] Frequency Wise
    - [ ] Pixel Wise
