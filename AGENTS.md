# AGENTS.md - Vera

This file provides high-signal context for OpenCode agents working on Vera (a privacy toolkit and image forensic analyzer written in Rust with a Slint GUI).

## Build & Run Commands

- **Build Release**: `cargo build --release`
- **Run Application**: 
  ```bash
  RUST_BACKTRACE=full SLINT_BACKEND=winit-software ./target/release/vera
  ```
- **Nix Development Shell**: `nix-shell` (provides all C libraries, llvm, cargo-nextest, etc.)

## Testing

- **Run all tests**: `cargo test` (powered by `cargo nextest`)

## Architecture & Toolchain Quirks

- **GUI Framework**: Slint (`v1.18.1`).
  - *Important*: Configured with `backend-winit-wayland` (**Wayland ONLY — no X11 support**) and `renderer-software`.
- **Theme Generation (`build.rs`)**:
  - Automatically parses SCSS theme files in `themes/` (e.g., `dark.scss`, `hotpink.scss`) and generates `OUT_DIR/generated_themes.rs`.
  - Adding or modifying a `.scss` file in `themes/` automatically triggers a rebuild via `cargo:rerun-if-changed`.
- **Core Processing Backends**:
  - `src/processing/exiv2_backend.rs`: EXIF metadata extraction and manipulation.
  - `src/processing/openssl_backend.rs`: AES-256 encryption/decryption.
  - `src/processing/steg_backend.rs`: Steganography (Metadata, LSB Pixel-wise, DCT Frequency-wise).
  - `src/processing/geo_utils.rs`: GPS coordinates conversion (Decimal Degrees, DMS, UTM, MGRS, etc.).
