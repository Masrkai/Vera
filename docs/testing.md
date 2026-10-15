# Testing Strategy in Vera

This document outlines the testing architecture, prioritization from must-have to nice-to-have, and rationale for testing choices in Vera.

## Testing Philosophy & Architecture

Vera relies on a hybrid testing approach combining:
1. **Rust Unit Tests**: For fast, isolated validation of mathematical calculations, cryptographic primitives, and data parsing logic.
2. **Rust Integration Tests**: For testing backend interactions (Exiv2 metadata manipulation, OpenSSL encryption/decryption, batch file operations) against real sample assets.
3. **Gherkin BDD (Behavior-Driven Development)**: Using the `cucumber` crate and `.feature` files under `tests/features/` to validate user-facing features and end-to-end workflows in plain language.

---

## Testing Priorities (Must-Have to Nice-to-Have)

### 1. Must-Have (Core Correctness & Data Integrity)
* **Geo Utilities & Coordinate Conversion**:
  * *Why*: Incorrect GPS parsing corrupts map location features.
  * *Implementation*: Rust unit tests (`tests/unit/geo_tests.rs`) covering conversion accuracy across Decimal Degrees, DMS, UTM, MGRS, Geohash, and Plus Codes, including edge cases (poles, equator, prime meridian).
* **EXIF Metadata Extraction & Modification**:
  * *Why*: Metadata reading and editing is Vera's core domain.
  * *Implementation*: Rust integration tests (`tests/integration/exif_tests.rs`) and Gherkin features (`tests/features/exif_extraction.feature`) verifying tag extraction, updates, and deletion using bundled sample images (`images/1.jpg`, `images/2.jpg`).
* **OpenSSL Encryption & Decryption**:
  * *Why*: Ensures data security and cryptographic reliability.
  * *Implementation*: Rust unit tests (`tests/unit/crypto_tests.rs`) testing round-trips and invalid passphrase handling.

### 2. Should-Have (Workflows & Batch Operations)
* **Batch Operations**:
  * *Why*: Multi-file batch edits/deletions must handle concurrent file states safely without corruption.
  * *Implementation*: Rust integration tests and Gherkin BDD features (`tests/features/batch_operations.feature`) processing temporary multi-file directories.
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
├── unit/
│   ├── geo_tests.rs
│   └── crypto_tests.rs
├── integration/
│   ├── exif_tests.rs
│   ├── batch_tests.rs
│   └── steg_tests.rs
├── features/
│   ├── exif_extraction.feature
│   ├── batch_operations.feature
│   └── forensic_workflow.feature
└── common/
    └── mod.rs
```

## Running Tests

*Note: Cargo commands are executed manually.*
To run cargo tests when implemented:
```bash
cargo test
```
To run cucumber BDD tests:
```bash
cargo test --test cucumber
```
