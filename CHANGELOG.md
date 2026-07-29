# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.5] - 2026-07-29

### Changed
- Rewrote `README.md`, `SECURITY.md`, and `CHANGELOG.md` in ASD-STE100
  strict mode. Removed marketing language, passive voice, and long
  sentences.
- Fixed `README.md` inaccuracies. The "Supported Types" section now lists
  the `[u8; N]` impls for `EncodeInto` and `TryDecodeFrom`. The "Error
  Types" section now lists `InsufficientData` and `InvalidEncoding`. The
  test count now reads 155.
- Fixed `SECURITY.md` inaccuracies. The "Bounded Operations" table now
  notes that `EncodeInto` for `[u8; N]` is unbounded. The "Version
  History" section now shows 1.0.5 as the current version. The
  best-practices examples no longer reference nonexistent error variants.
- Updated `README.md` install version from `1.0` to `1.1`.

## [1.0.4] - 2026-07-16

### Security
- Removed the unmaintained `serde_cbor` dev-dependency. This addresses
  RUSTSEC-2021-0127.

### Changed
- Removed the redundant `unsafe impl Send/Sync for EncodedBytes`. The
  type gets `Send` and `Sync` from its `Vec<u8>` field. The crate
  `#![deny(unsafe_code)]` policy now holds without exception.
- Updated `SECURITY.md` to state the zero-unsafe claim correctly.

## [1.0.3] - 2026-07-15

### Added

- `#[inline]` on hot encode and decode paths. This covers `EncodeInto`
  impls for all integer types and `bool`. It covers `TryDecodeFrom`
  impls for all integer types, `bool`, and fixed-length arrays.
- MSRV declared as `rust-version = "1.85"` in `Cargo.toml`. CI verifies
  the MSRV with a dedicated job.
- `cargo audit` job in CI.
- `cargo fmt --check` and `clippy -D warnings` steps in CI.
- `no_std` build verification job in CI.
- Clippy lint configuration. `[lints.clippy]` sets `pedantic`, `nursery`,
  and `cargo` groups to `warn`. `[lints.rust]` sets `unsafe_code` to
  `deny`.
- Length-bounds documentation on the `TryDecodeFrom` trait. The doc notes
  that integer decoders rely on `unsigned-varint` byte limits. It advises
  callers that decode length-prefixed payloads to set their own upper
  bound.

### Changed

- Edition updated to 2024 from 2021.
- Clippy pedantic, nursery, and cargo warnings resolved across all
  source, examples, and benchmarks.

## [1.0.0] - 2026-07-13

### Changed
- Synced from the bettersign workspace (`bs-multitrait` 0.7.0).
- Renamed the crate from `bs-multitrait` to `multi-trait`.
- Initial published release on crates.io as `multi-trait`.

## [1.0.1] - 2024-08-27

### Fixed
- Fixed the `no_std` build. Added `alloc` imports to all modules.
- Fixed error handling in `no_std` mode. The `UnsignedVarintDecode`
  error variant is now conditionally compiled.
- Removed unused imports found by clippy.

### Changed
- Improved code quality with clippy pedantic lints.
- Added numeric literal separators for readability.
- Changed `assert_eq!(val, true)` to `assert!(val)` for idiomatic code.
- Changed indexing loops to iterators where applicable.

### Documentation
- Improved documentation formatting.
- Added error handling examples.
- Improved module-level documentation across all traits.

## [bs-multitrait 1.0.0] - 2024-04-14

### Added
- Initial stable release as `bs-multitrait`.
- Core encoding traits: `EncodeInto`, `EncodeIntoBuffer`,
  `EncodeIntoArray`.
- Core decoding trait: `TryDecodeFrom`.
- Null value traits: `Null` and `TryNull`.
- Validated newtype: `EncodedBytes` for type-safe varint-encoded byte
  sequences.
- Error handling with the `Error` enum.
- Full `no_std` support with `alloc`.
- Thread-safe implementations (`Send + Sync`).
- Test suite with 105 tests: unit, property-based, security,
  concurrency, and round-trip tests.

### Performance
- Zero-copy decoding with slice references.
- Zero-allocation buffer encoding.
- Stack-only encoding for embedded systems.
- Single-allocation encoding for heap operations.

### Documentation
- Module and trait documentation.
- Usage examples for all public APIs.
- Performance characteristics documentation.
- Thread safety documentation.

[1.0.5]: https://github.com/cryptidtech/multi-trait/compare/v1.0.4...v1.0.5
[1.0.4]: https://github.com/cryptidtech/multi-trait/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/cryptidtech/multi-trait/compare/v1.0.1...v1.0.3
[1.0.0]: https://github.com/cryptidtech/multi-trait/releases/tag/v1.0.0
[1.0.1]: https://github.com/cryptidtech/multi-trait/compare/v1.0.0...v1.0.1
[bs-multitrait 1.0.0]: https://github.com/cryptidtech/multi-trait/releases/tag/bs-multitrait-v1.0.0