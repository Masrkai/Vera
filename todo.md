# TODO List

- [x] The basic main goal of making a drop-in Google Maps viewer using image metadata, while simultaneously being able to see, edit, or delete metadata in single or multiple batches.

## Implemented Features & Criteria

### 1. EXIF Metadata Management & Visualization
- [x] Single Image Workspace
  - [x] EXIF extraction and parsing (`exiv2_backend.rs`)
  - [x] Tag filtering and searching
  - [x] Inline editing of EXIF tag values
  - [x] Tag deletion (individual tags)
  - [x] Complete EXIF metadata deletion with confirmation dialog
  - [x] Preserve original file option
- [x] Batch Processing Workspace
  - [x] Queue management for multiple images (`batch_operations.rs`)
  - [x] Batch EXIF deletion
  - [x] Batch progress tracking and status results
  - [x] Preserve originals option for batch operations

### 2. Geolocation & Mapping
- [x] Drop-in Google Maps viewer using image GPS metadata (`geo_utils.rs`)
- [x] Alternative ESRI satellite imagery viewer option

### 3. Forensic Analysis, Cryptography & Steganography
- [x] AES-256 Encryption & Decryption (`openssl_backend.rs`)
- [x] Steganography Embedding & Extraction (`steg_backend.rs`)
  - [x] Metadata (EXIF) Wise
  - [x] Pixel Wise (LSB - Least Significant Bit)
  - [x] Frequency Wise (DCT - Discrete Cosine Transform)
- [x] BDD Testing Suite (Cucumber feature tests for exif extraction, batch operations, and forensic workflow)

### 4. UI & Theming
- [x] Responsive Slint GUI (`ui/main.slint`, `components.slint`)
- [x] Multi-theme support (Dark, Light, Gruvbox, Dracula, Nord, Hotpink)
- [x] Image preview and maximized image modal viewer

### 5. Integration & Packaging
- [x] XDG desktop integration ("Open with" / `.desktop` file support)
- [x] App icon asset (`assets/Vera.svg`)
- [x] NixOS support & development shell (`vera.nix`, `shell.nix`)



