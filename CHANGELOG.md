# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.3] - 2026-07-15

### Added

- **`#[inline]`** on hot encode/decode paths: `EncodeInto` impls for all
  integer types and `bool`, `TryDecodeFrom` impls for all integer types,
  `bool`, and fixed-length arrays.
- **MSRV declared**: `rust-version = "1.85"` in `Cargo.toml`. CI verifies the
  MSRV with a dedicated job.
- **`cargo audit`** job in CI.
- **`cargo fmt --check`** and **`clippy -D warnings`** steps in CI.
- **`no_std` build verification** job in CI.
- **Clippy lint configuration**: `[lints.clippy]` with `pedantic`, `nursery`,
  and `cargo` groups (all `warn`), plus `[lints.rust] unsafe_code = "deny"`.
- **Length-bounds documentation** on the `TryDecodeFrom` trait noting that
  integer decoders rely on `unsigned-varint`'s type-specific byte limits and
  that callers decoding length-prefixed payloads should enforce their own
  upper bound.

### Changed

- **Edition 2024**: Updated from Rust 2021.
- **Clippy pedantic/nursery/cargo warnings** resolved across all source,
  examples, and benchmarks.

## [1.0.0] - 2026-07-13

### Changed
- Synced from bettersign workspace (bs-multitrait 0.7.0)
- Renamed crate from `bs-multitrait` to `multi-trait`
- Initial published release on crates.io as `multi-trait`

## [1.0.1] - 2024-08-27

### Fixed
- Fixed `no_std` build by adding proper `alloc` imports to all modules
- Fixed error handling in `no_std` mode by conditionally compiling `UnsignedVarintDecode` error variant
- Removed unused imports identified by clippy

### Changed
- Improved code quality with clippy pedantic lints
- Enhanced code readability by adding numeric literal separators (e.g., `100_000` instead of `100000`)
- Improved documentation formatting with proper backticks for code items
- Updated bool assertions from `assert_eq!(val, true)` to `assert!(val)` for better idiomatic code
- Optimized loop patterns from indexing to iterators where applicable

### Documentation
- Enhanced documentation with better formatting
- Added comprehensive error handling examples
- Improved module-level documentation across all traits
- Verified all documentation links work correctly

## [bs-multitrait 1.0.0] - 2024-04-14

### Added
- Initial stable release (as `bs-multitrait`, the bettersign workspace name)
- Core encoding traits: `EncodeInto`, `EncodeIntoBuffer`, `EncodeIntoArray`
- Core decoding trait: `TryDecodeFrom`
- Null value traits: `Null` and `TryNull`
- Validated newtype: `EncodedBytes` for type-safe varint-encoded byte sequences
- Comprehensive error handling with `Error` enum
- Full `no_std` support with `alloc`
- Thread-safe implementations (Send + Sync)
- Extensive test suite with 105 tests including:
  - Unit tests for all traits
  - Property-based tests with proptest
  - Security tests for malicious input handling
  - Concurrency tests for multi-threaded usage
  - Round-trip encoding/decoding tests

### Performance
- Zero-copy decoding with slice references
- Zero-allocation buffer encoding
- Stack-only encoding for embedded systems
- Single-allocation encoding for heap operations
- Optimized varint format for space efficiency

### Documentation
- Comprehensive module and trait documentation
- Usage examples for all public APIs
- Performance characteristics documentation
- Thread safety guarantees documented
- Security considerations documented

[1.0.3]: https://github.com/cryptidtech/multi-trait/compare/v1.0.1...v1.0.3
[1.0.0]: https://github.com/cryptidtech/multi-trait/releases/tag/v1.0.0
[1.0.1]: https://github.com/cryptidtech/multi-trait/compare/v1.0.0...v1.0.1
[bs-multitrait 1.0.0]: https://github.com/cryptidtech/multi-trait/releases/tag/bs-multitrait-v1.0.0