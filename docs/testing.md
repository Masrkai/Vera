# Testing Strategy in Vera

This document outlines the testing architecture, prioritization from must-have to nice-to-have, and rationale for testing choices in Vera.

## Testing Philosophy & Architecture

Vera relies on a robust testing approach combining:
1. **Rust Unit Tests**: For fast, isolated validation of mathematical calculations, cryptographic primitives, and data parsing logic.
2. **Rust Integration Tests**: For testing backend interactions (Exiv2 metadata manipulation, OpenSSL encryption/decryption, batch file operations) against real sample assets.

---

## Testing Priorities (Must-Have to Nice-to-Have)

### 1. Must-Have (Core Correctness & Data Integrity)
* **Geo Utilities & Coordinate Conversion**:
  * *Why*: Incorrect GPS parsing corrupts map location features.
  * *Implementation*: Rust unit tests (`tests/unit/geo_tests.rs`) covering conversion accuracy across Decimal Degrees, DMS, UTM, MGRS, Geohash, and Plus Codes, including edge cases (poles, equator, prime meridian).
* **EXIF Metadata Extraction & Modification**:
  * *Why*: Metadata reading and editing is Vera's core domain.
  * *Implementation*: Rust integration tests (`tests/integration/exif_tests.rs`) verifying tag extraction, updates, and deletion using bundled sample images (`images/1.jpg`, `images/2.jpg`).
* **OpenSSL Encryption & Decryption**:
  * *Why*: Ensures data security and cryptographic reliability.
  * *Implementation*: Rust unit tests (`tests/unit/crypto_tests.rs`) testing round-trips and invalid passphrase handling.

### 2. Should-Have (Workflows & Batch Operations)
* **Batch Operations**:
  * *Why*: Multi-file batch edits/deletions must handle concurrent file states safely without corruption.
  * *Implementation*: Rust integration tests (`tests/integration/batch_tests.rs`) processing multi-file sample sets.
* **Steganography Embedding & Extraction**:
  * *Why*: Forensic hiding requires precise bit/pixel/frequency manipulation.
  * *Implementation*: Rust integration tests (`tests/integration/steg_tests.rs`) covering pixel-wise and frequency-wise embedding methods.

### 3. Nice-to-Have (Edge Cases & Advanced Forensics)
* **Backend Dependency & Error Handling**:
  * *Why*: Graceful degradation when external binaries (`exiv2`, OpenSSL) are absent.
* **End-to-End Forensic Workflows**:
  * *Why*: Validating the complete lifecycle from encryption to steganographic embedding and extraction.

---

## Directory Structure

```text
tests/
├── geo_tests.rs
├── crypto_tests.rs
├── exif_tests.rs
├── batch_tests.rs
├── steg_tests.rs
└── common/
    └── mod.rs
```

## Running Tests

Vera uses `cargo nextest` for fast, parallelized test execution. Via Cargo configuration (`.cargo/config.toml`), running standard cargo test commands automatically invokes `nextest`:

```bash
cargo test
```

Or run nextest explicitly:
```bash
cargo nextest run
```
