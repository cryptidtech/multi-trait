// SPDX-License-Identifier: Apache-2.0
//! # Multitrait
//!
//! A lightweight, high-performance library providing common traits for implementing
//! [multiformats](https://github.com/multiformats/multiformats) types in Rust.
//!
//! ## Overview
//!
//! This crate provides core traits that standardize encoding, decoding, and
//! null value handling across multiformats implementations:
//!
//! ### Encoding Traits
//!
//! - **[`EncodeInto`]**: Encode values into compact varint `Vec<u8>` format
//! - **[`EncodeIntoBuffer`]**: Zero-allocation encoding into existing buffers
//! - **[`EncodeIntoArray`]**: Stack-based encoding for `no_std` environments
//!
//! ### Decoding Traits
//!
//! - **[`TryDecodeFrom`]**: Fallibly decode values from byte slices with remainder tracking
//!
//! ### Null Value Traits
//!
//! - **[`Null`]**: Define and check for null/default values
//! - **[`TryNull`]**: Fallible version of `Null` for types requiring validation
//!
//! ### Validated Types
//!
//! - **[`EncodedBytes`]**: Validated newtype for varint-encoded byte sequences
//!
//! ## Features
//!
//! - **Zero-copy decoding**: `TryDecodeFrom` returns remaining bytes without allocation
//! - **Zero-allocation encoding**: `EncodeIntoBuffer` reuses existing buffers
//! - **Stack-based encoding**: `EncodeIntoArray` for `embedded/no_std` contexts
//! - **Optimized encoding**: Single-allocation encoding with efficient varint compression
//! - **`no_std` support**: Works in embedded and constrained environments (with `alloc`)
//! - **Type-safe errors**: Structured error types with proper error chains
//! - **Thread-safe**: All traits are `Send + Sync` safe
//!
//! ## Quick Start
//!
//! ```rust
//! use multi_trait::{EncodeInto, TryDecodeFrom};
//!
//! // Encoding: Convert a value to compact varint bytes
//! let value = 42u32;
//! let encoded = value.encode_into();
//! println!("Encoded {} as {:?}", value, encoded);
//!
//! // Decoding: Parse bytes back to original value
//! let (decoded, remaining) = u32::try_decode_from(&encoded).unwrap();
//! assert_eq!(decoded, value);
//! assert!(remaining.is_empty());
//! ```
//!
//! ## Encoding Example
//!
//! The [`EncodeInto`] trait provides efficient varint encoding:
//!
//! ```rust
//! use multi_trait::EncodeInto;
//!
//! // Small values use fewer bytes
//! assert_eq!(0u8.encode_into(), vec![0]);
//! assert_eq!(127u8.encode_into(), vec![127]);
//! assert_eq!(128u8.encode_into(), vec![128, 1]); // Requires 2 bytes
//!
//! // Works with all unsigned integer types
//! let large_value = 0xFFFF_FFFF_u32;
//! let encoded = large_value.encode_into();
//! println!("Encoded 0x{:X} in {} bytes", large_value, encoded.len());
//! ```
//!
//! ## Decoding Example
//!
//! The [`TryDecodeFrom`] trait enables zero-copy parsing with error handling:
//!
//! ```rust
//! use multi_trait::TryDecodeFrom;
//!
//! // Decode from byte slice
//! let bytes = vec![0xFF, 0xFF, 0x03]; // Varint encoding of 65535
//! let (value, remaining) = u16::try_decode_from(&bytes).unwrap();
//! assert_eq!(value, 65535);
//! assert!(remaining.is_empty());
//!
//! // Handle errors gracefully
//! let empty: &[u8] = &[];
//! let result = u8::try_decode_from(empty);
//! assert!(result.is_err());
//! ```
//!
//! ## Null Value Handling
//!
//! Define sentinel/null values for custom types:
//!
//! ```rust
//! use multi_trait::Null;
//!
//! struct MyId(u64);
//!
//! impl Null for MyId {
//!     fn null() -> Self {
//!         MyId(0)
//!     }
//!
//!     fn is_null(&self) -> bool {
//!         self.0 == 0
//!     }
//! }
//!
//! let null_id = MyId::null();
//! assert!(null_id.is_null());
//!
//! let valid_id = MyId(12345);
//! assert!(!valid_id.is_null());
//! ```
//!
//! ## Error Handling
//!
//! All decode operations return a [`Result`] with a structured [`Error`] type:
//!
//! ```rust
//! use multi_trait::{TryDecodeFrom, Error};
//!
//! let truncated = vec![0xFF]; // Incomplete varint
//! match u16::try_decode_from(&truncated) {
//!     Ok((value, _)) => println!("Decoded: {}", value),
//!     Err(Error::UnsignedVarintDecode { source }) => {
//!         eprintln!("Decode failed: {}", source);
//!     }
//!     Err(e) => eprintln!("Other error: {}", e),
//! }
//! ```
//!
//! ## Buffer-Based Encoding (Zero Allocation)
//!
//! The [`EncodeIntoBuffer`] trait enables encoding without allocations:
//!
//! ```rust
//! use multi_trait::EncodeIntoBuffer;
//!
//! // Create a reusable buffer
//! let mut buffer = Vec::with_capacity(100);
//!
//! // Encode multiple values with no additional allocations
//! 42u8.encode_into_buffer(&mut buffer);
//! 1000u16.encode_into_buffer(&mut buffer);
//! 100000u32.encode_into_buffer(&mut buffer);
//!
//! // All three values encoded in one buffer
//! println!("Encoded {} bytes", buffer.len());
//! ```
//!
//! ## Stack-Based Encoding (No Heap)
//!
//! The [`EncodeIntoArray`] trait provides stack-only encoding for embedded systems:
//!
//! ```rust
//! use multi_trait::EncodeIntoArray;
//!
//! // Encode to stack-allocated array (no heap)
//! let (array, len) = 42u8.encode_into_array();
//! assert_eq!(&array[..len], &[42]);
//!
//! // Maximum sizes known at compile time
//! assert_eq!(<u32 as EncodeIntoArray>::MAX_ENCODED_SIZE, 5);
//! ```
//!
//! ## Type Safety with Validated Newtypes
//!
//! The [`EncodedBytes`] newtype provides compile-time guarantees that bytes
//! represent valid varint encodings:
//!
//! ```rust
//! use multi_trait::EncodedBytes;
//!
//! // Validation happens at construction
//! let valid = vec![42u8];
//! let encoded = EncodedBytes::try_from(valid).unwrap();
//!
//! // Invalid data is rejected
//! let invalid = vec![0x80]; // Truncated varint
//! assert!(EncodedBytes::try_from(invalid).is_err());
//!
//! // Type system ensures valid data
//! fn process_encoded(data: EncodedBytes) {
//!     // No need to validate - type guarantees validity
//!     println!("Processing {} bytes", data.len());
//! }
//! ```
//!
//! ## Performance Characteristics
//!
//! - **`EncodeInto`**: Single allocation, O(1) complexity for finding varint length
//! - **`EncodeIntoBuffer`**: Zero allocations (reuses buffer capacity), ideal for hot paths
//! - **`EncodeIntoArray`**: Zero heap allocations (stack only), deterministic performance
//! - **`TryDecodeFrom`**: Zero allocations, returns slice references
//! - **Varint format**: Compact representation, 1-10 bytes per integer depending on value
//!
//! ## Thread Safety
//!
//! All traits and types in this crate are `Send + Sync`, making them safe to use
//! in concurrent contexts. This section documents the thread-safety guarantees.
//!
//! ### Trait Implementations
//!
//! All trait implementations (`EncodeInto`, `TryDecodeFrom`, `Null`, `TryNull`)
//! are stateless and immutable, providing these guarantees:
//!
//! - **`Send`**: Values can be transferred between threads
//! - **`Sync`**: References can be shared between threads
//! - **No locks required**: All operations are lock-free
//! - **No data races**: No mutable state is shared
//!
//! ### Type Safety
//!
//! The [`EncodedBytes`] newtype is explicitly marked as `Send + Sync`:
//!
//! ```rust
//! use multi_trait::EncodedBytes;
//! use std::sync::Arc;
//! use std::thread;
//!
//! let encoded = EncodedBytes::new(&[42]).unwrap();
//! let shared = Arc::new(encoded);
//!
//! // Can be shared across threads safely
//! let handles: Vec<_> = (0..4)
//!     .map(|_| {
//!         let data = Arc::clone(&shared);
//!         thread::spawn(move || {
//!             assert_eq!(&data[..], &[42]);
//!         })
//!     })
//!     .collect();
//!
//! for handle in handles {
//!     handle.join().unwrap();
//! }
//! ```
//!
//! ### Concurrency Patterns
//!
//! Common patterns that work safely:
//!
//! - **Parallel encoding**: Multiple threads can encode different values simultaneously
//! - **Shared decoding**: Multiple threads can decode from the same source data
//! - **Pipeline processing**: Encode in one thread, decode in another
//! - **Work stealing**: Tasks can move between threads freely
//!
//! ## Feature Flags
//!
//! - **`std`** (default): Enables standard library support
//!   - Disable for `no_std` environments: `default-features = false`
//!   - Requires `alloc` when disabled (for `Vec<u8>` support)
//!
//! ## no_std Support
//!
//! This crate works in `no_std` environments with `alloc`:
//!
//! ```toml
//! [dependencies]
//! multitrait = { version = "1.0", default-features = false }
//! ```
//!
//! ## Implementation Details
//!
//! The crate uses production-quality declarative macros to eliminate code duplication
//! while maintaining zero runtime overhead. All encoding/decoding implementations
//! are generated at compile time with full type safety.
#![warn(missing_docs)]
#![deny(
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

/// Errors generated from the implementations
pub mod error;
pub use error::Error;

/// EncodeInto trait
pub mod enc_into;
pub use enc_into::EncodeInto;

/// EncodeIntoBuffer trait for zero-allocation encoding
pub mod enc_into_buffer;
pub use enc_into_buffer::EncodeIntoBuffer;

/// EncodeIntoArray trait for stack-based encoding
pub mod enc_into_array;
pub use enc_into_array::EncodeIntoArray;

/// Null and TryNull traits
pub mod null;
pub use null::{Null, TryNull};

/// TryDecodeFrom trait
pub mod try_decode_from;
pub use try_decode_from::TryDecodeFrom;

/// Validated newtype for encoded bytes
pub mod encoded_bytes;
pub use encoded_bytes::EncodedBytes;

/// one-stop shop for all exported symbols
pub mod prelude {
    pub use super::{
        enc_into::*, enc_into_array::*, enc_into_buffer::*, encoded_bytes::*, null::*,
        try_decode_from::*,
    };
}

#[cfg(test)]
mod test {
    use super::prelude::*;

    #[test]
    fn test_bool() {
        let tbuf = true.encode_into();
        let (tval, _) = bool::try_decode_from(&tbuf).unwrap();
        assert!(tval);
        let fbuf = false.encode_into();
        let (fval, _) = bool::try_decode_from(&fbuf).unwrap();
        assert!(!fval);
    }

    #[test]
    fn test_u8() {
        let buf = 0xff_u8.encode_into();
        let (num, _) = u8::try_decode_from(&buf).unwrap();
        assert_eq!(0xff_u8, num);
    }

    #[test]
    fn test_u16() {
        let buf = 0xffee_u16.encode_into();
        let (num, _) = u16::try_decode_from(&buf).unwrap();
        assert_eq!(0xffee_u16, num);
    }

    #[test]
    fn test_u32() {
        let buf = 0xffee_ddcc_u32.encode_into();
        let (num, _) = u32::try_decode_from(&buf).unwrap();
        assert_eq!(0xffee_ddcc_u32, num);
    }

    #[test]
    fn test_u64() {
        let buf = 0xffee_ddcc_bbaa_9988_u64.encode_into();
        let (num, _) = u64::try_decode_from(&buf).unwrap();
        assert_eq!(0xffee_ddcc_bbaa_9988_u64, num);
    }

    #[test]
    fn test_u128() {
        let buf = 0xffee_ddcc_bbaa_9988_7766_5544_3322_1100_u128.encode_into();
        let (num, _) = u128::try_decode_from(&buf).unwrap();
        assert_eq!(0xffee_ddcc_bbaa_9988_7766_5544_3322_1100_u128, num);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_usize() {
        let buf = 0xffee_ddcc_bbaa_9988_usize.encode_into();
        let (num, _) = usize::try_decode_from(&buf).unwrap();
        assert_eq!(0xffee_ddcc_bbaa_9988_usize, num);
    }

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_usize() {
        let buf = 0xffeeddcc_usize.encode_into();
        let (num, _) = usize::try_decode_from(&buf).unwrap();
        assert_eq!(0xffeeddcc_usize, num);
    }

    struct Foo(usize);

    impl Null for Foo {
        fn null() -> Self {
            Foo(0)
        }
        fn is_null(&self) -> bool {
            self.0 == 0
        }
    }

    impl TryNull for Foo {
        type Error = &'static str;

        fn try_null() -> Result<Self, Self::Error> {
            Ok(Foo(0))
        }
        fn is_null(&self) -> bool {
            self.0 == 0
        }
    }

    #[test]
    fn test_null_value() {
        let f = Foo::null();
        assert!(Null::is_null(&f));
    }

    #[test]
    fn test_try_null_value() {
        let f = Foo::try_null().unwrap();
        assert!(TryNull::is_null(&f));
    }

    // ========================================================================
    // Error Case Tests
    // ========================================================================

    #[test]
    fn test_decode_empty_slice_u8() {
        let empty: &[u8] = &[];
        let result = u8::try_decode_from(empty);
        assert!(result.is_err(), "Should fail to decode from empty slice");
    }

    #[test]
    fn test_decode_empty_slice_u16() {
        let empty: &[u8] = &[];
        let result = u16::try_decode_from(empty);
        assert!(result.is_err(), "Should fail to decode from empty slice");
    }

    #[test]
    fn test_decode_empty_slice_u32() {
        let empty: &[u8] = &[];
        let result = u32::try_decode_from(empty);
        assert!(result.is_err(), "Should fail to decode from empty slice");
    }

    #[test]
    fn test_decode_empty_slice_u64() {
        let empty: &[u8] = &[];
        let result = u64::try_decode_from(empty);
        assert!(result.is_err(), "Should fail to decode from empty slice");
    }

    #[test]
    fn test_decode_empty_slice_u128() {
        let empty: &[u8] = &[];
        let result = u128::try_decode_from(empty);
        assert!(result.is_err(), "Should fail to decode from empty slice");
    }

    #[test]
    fn test_decode_empty_slice_bool() {
        let empty: &[u8] = &[];
        let result = bool::try_decode_from(empty);
        assert!(result.is_err(), "Should fail to decode from empty slice");
    }

    #[test]
    fn test_decode_truncated_varint() {
        // A varint with continuation bit set but no following byte
        let truncated = vec![0x80]; // MSB set, indicates more bytes follow
        let result = u16::try_decode_from(&truncated);
        assert!(result.is_err(), "Should fail on truncated varint");
    }

    #[test]
    fn test_decode_truncated_large_varint() {
        // Incomplete multi-byte varint
        let truncated = vec![0xFF, 0xFF]; // Two bytes with continuation bits
        let result = u32::try_decode_from(&truncated);
        assert!(result.is_err(), "Should fail on truncated large varint");
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================

    #[test]
    fn test_encode_decode_zero_values() {
        // Test zero value for all types
        assert_eq!(0u8.encode_into(), vec![0]);
        assert_eq!(0u16.encode_into(), vec![0]);
        assert_eq!(0u32.encode_into(), vec![0]);
        assert_eq!(0u64.encode_into(), vec![0]);
        assert_eq!(0u128.encode_into(), vec![0]);
        assert_eq!(0usize.encode_into(), vec![0]);

        // Verify decode
        let (val, rest) = u8::try_decode_from(&[0]).unwrap();
        assert_eq!(val, 0);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_encode_decode_max_u8() {
        let max = u8::MAX;
        let encoded = max.encode_into();
        let (decoded, remaining) = u8::try_decode_from(&encoded).unwrap();
        assert_eq!(decoded, max);
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_encode_decode_max_u16() {
        let max = u16::MAX;
        let encoded = max.encode_into();
        let (decoded, remaining) = u16::try_decode_from(&encoded).unwrap();
        assert_eq!(decoded, max);
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_encode_decode_max_u32() {
        let max = u32::MAX;
        let encoded = max.encode_into();
        let (decoded, remaining) = u32::try_decode_from(&encoded).unwrap();
        assert_eq!(decoded, max);
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_encode_decode_max_u64() {
        let max = u64::MAX;
        let encoded = max.encode_into();
        let (decoded, remaining) = u64::try_decode_from(&encoded).unwrap();
        assert_eq!(decoded, max);
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_encode_decode_max_u128() {
        let max = u128::MAX;
        let encoded = max.encode_into();
        let (decoded, remaining) = u128::try_decode_from(&encoded).unwrap();
        assert_eq!(decoded, max);
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_varint_boundary_127() {
        // 127 should encode in 1 byte (last value that fits in 7 bits)
        let val = 127u8;
        let encoded = val.encode_into();
        assert_eq!(encoded.len(), 1);
        let (decoded, _) = u8::try_decode_from(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn test_varint_boundary_128() {
        // 128 should require 2 bytes (first value needing continuation)
        let val = 128u8;
        let encoded = val.encode_into();
        assert_eq!(encoded.len(), 2);
        let (decoded, _) = u8::try_decode_from(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn test_bool_nonzero_as_true() {
        // Any non-zero value should decode as true
        let bytes = vec![42];
        let (val, _) = bool::try_decode_from(&bytes).unwrap();
        assert!(val);
    }

    // ========================================================================
    // Round-Trip Tests
    // ========================================================================

    #[test]
    fn test_roundtrip_u8_range() {
        // Test a range of u8 values
        for value in [0, 1, 127, 128, 255] {
            let encoded = value.encode_into();
            let (decoded, remaining) = u8::try_decode_from(&encoded).unwrap();
            assert_eq!(decoded, value, "Round-trip failed for u8 value {}", value);
            assert!(remaining.is_empty());
        }
    }

    #[test]
    fn test_roundtrip_u16_range() {
        // Test various u16 values including boundaries
        for value in [0, 1, 127, 128, 255, 256, 16383, 16384, 65535] {
            let encoded = value.encode_into();
            let (decoded, remaining) = u16::try_decode_from(&encoded).unwrap();
            assert_eq!(decoded, value, "Round-trip failed for u16 value {}", value);
            assert!(remaining.is_empty());
        }
    }

    #[test]
    fn test_roundtrip_u32_range() {
        // Test various u32 values
        for value in [0, 1, 127, 128, 16384, 65536, u32::MAX] {
            let encoded = value.encode_into();
            let (decoded, remaining) = u32::try_decode_from(&encoded).unwrap();
            assert_eq!(decoded, value, "Round-trip failed for u32 value {}", value);
            assert!(remaining.is_empty());
        }
    }

    #[test]
    fn test_roundtrip_u64_range() {
        // Test various u64 values
        for value in [0, 1, 127, 128, 65536, u32::MAX as u64, u64::MAX] {
            let encoded = value.encode_into();
            let (decoded, remaining) = u64::try_decode_from(&encoded).unwrap();
            assert_eq!(decoded, value, "Round-trip failed for u64 value {}", value);
            assert!(remaining.is_empty());
        }
    }

    #[test]
    fn test_roundtrip_bool() {
        for value in [true, false] {
            let encoded = value.encode_into();
            let (decoded, remaining) = bool::try_decode_from(&encoded).unwrap();
            assert_eq!(decoded, value, "Round-trip failed for bool {}", value);
            assert!(remaining.is_empty());
        }
    }

    #[test]
    fn test_sequential_decode() {
        // Encode multiple values into one buffer
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&42u8.encode_into());
        buffer.extend_from_slice(&1000u16.encode_into());
        buffer.extend_from_slice(&100_000_u32.encode_into());

        // Decode sequentially
        let (val1, rest) = u8::try_decode_from(&buffer).unwrap();
        assert_eq!(val1, 42);

        let (val2, rest) = u16::try_decode_from(rest).unwrap();
        assert_eq!(val2, 1000);

        let (val3, rest) = u32::try_decode_from(rest).unwrap();
        assert_eq!(val3, 100_000);

        assert!(rest.is_empty(), "Should have consumed all bytes");
    }

    #[test]
    fn test_remaining_bytes_returned() {
        // Verify that decode returns unconsumed bytes
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&42u8.encode_into());
        buffer.extend_from_slice(&[0xFF, 0xEE, 0xDD]); // Extra bytes

        let (val, remaining) = u8::try_decode_from(&buffer).unwrap();
        assert_eq!(val, 42);
        assert_eq!(remaining, &[0xFF, 0xEE, 0xDD]);
    }

    #[test]
    fn test_encode_length_efficiency() {
        // Verify varint encoding is space-efficient
        assert_eq!(0u8.encode_into().len(), 1);
        assert_eq!(127u8.encode_into().len(), 1);
        assert_eq!(128u8.encode_into().len(), 2);
        assert_eq!(255u8.encode_into().len(), 2);

        // Larger values should use more bytes
        assert!(u16::MAX.encode_into().len() > 1);
        assert!(u32::MAX.encode_into().len() > 2);
    }

    // ========================================================================
    // Property-Based Tests (using proptest)
    // ========================================================================

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_roundtrip_u8(value: u8) {
            let encoded = value.encode_into();
            let (decoded, remaining) = u8::try_decode_from(&encoded).unwrap();
            prop_assert_eq!(decoded, value);
            prop_assert!(remaining.is_empty());
        }

        #[test]
        fn prop_roundtrip_u16(value: u16) {
            let encoded = value.encode_into();
            let (decoded, remaining) = u16::try_decode_from(&encoded).unwrap();
            prop_assert_eq!(decoded, value);
            prop_assert!(remaining.is_empty());
        }

        #[test]
        fn prop_roundtrip_u32(value: u32) {
            let encoded = value.encode_into();
            let (decoded, remaining) = u32::try_decode_from(&encoded).unwrap();
            prop_assert_eq!(decoded, value);
            prop_assert!(remaining.is_empty());
        }

        #[test]
        fn prop_roundtrip_u64(value: u64) {
            let encoded = value.encode_into();
            let (decoded, remaining) = u64::try_decode_from(&encoded).unwrap();
            prop_assert_eq!(decoded, value);
            prop_assert!(remaining.is_empty());
        }

        #[test]
        fn prop_roundtrip_u128(value: u128) {
            let encoded = value.encode_into();
            let (decoded, remaining) = u128::try_decode_from(&encoded).unwrap();
            prop_assert_eq!(decoded, value);
            prop_assert!(remaining.is_empty());
        }

        #[test]
        fn prop_roundtrip_usize(value: usize) {
            let encoded = value.encode_into();
            let (decoded, remaining) = usize::try_decode_from(&encoded).unwrap();
            prop_assert_eq!(decoded, value);
            prop_assert!(remaining.is_empty());
        }

        #[test]
        fn prop_roundtrip_bool(value: bool) {
            let encoded = value.encode_into();
            let (decoded, remaining) = bool::try_decode_from(&encoded).unwrap();
            prop_assert_eq!(decoded, value);
            prop_assert!(remaining.is_empty());
        }

        #[test]
        fn prop_encode_not_empty(value: u32) {
            let encoded = value.encode_into();
            prop_assert!(!encoded.is_empty());
        }

        #[test]
        fn prop_encode_deterministic(value: u64) {
            let encoded1 = value.encode_into();
            let encoded2 = value.encode_into();
            prop_assert_eq!(encoded1, encoded2);
        }

        #[test]
        fn prop_small_values_compact(value in 0u32..128) {
            let encoded = value.encode_into();
            prop_assert_eq!(encoded.len(), 1, "Values under 128 should encode in 1 byte");
        }

        #[test]
        fn prop_sequential_decode(v1: u8, v2: u16, v3: u32) {
            let mut buffer = Vec::new();
            buffer.extend_from_slice(&v1.encode_into());
            buffer.extend_from_slice(&v2.encode_into());
            buffer.extend_from_slice(&v3.encode_into());

            let (d1, rest) = u8::try_decode_from(&buffer).unwrap();
            prop_assert_eq!(d1, v1);

            let (d2, rest) = u16::try_decode_from(rest).unwrap();
            prop_assert_eq!(d2, v2);

            let (d3, rest) = u32::try_decode_from(rest).unwrap();
            prop_assert_eq!(d3, v3);

            prop_assert!(rest.is_empty());
        }
    }

    // ========================================================================
    // Thread-Safety Tests (Compile-Time Verification)
    // ========================================================================

    /// Compile-time verification that Error is Send + Sync
    #[test]
    fn assert_error_send_sync() {
        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}
        is_send::<crate::Error>();
        is_sync::<crate::Error>();
    }

    /// Compile-time verification that EncodedBytes is Send + Sync
    #[test]
    fn assert_encoded_bytes_send_sync() {
        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}
        is_send::<EncodedBytes>();
        is_sync::<EncodedBytes>();
    }

    #[test]
    fn test_encoded_bytes_valid() {
        let bytes = vec![42];
        let encoded = EncodedBytes::try_from(bytes).unwrap();
        assert_eq!(encoded.as_ref(), &[42]);
    }

    #[test]
    fn test_encoded_bytes_invalid_empty() {
        let empty: Vec<u8> = vec![];
        assert!(EncodedBytes::try_from(empty).is_err());
    }

    #[test]
    fn test_encoded_bytes_invalid_truncated() {
        let truncated = vec![0x80];
        assert!(EncodedBytes::try_from(truncated).is_err());
    }

    #[test]
    fn test_encoded_bytes_into_vec() {
        // Use a single varint encoding (128 requires 2 bytes: [0x80, 0x01])
        let original = 128u8.encode_into();
        let encoded = EncodedBytes::new(&original).unwrap();
        let recovered: Vec<u8> = encoded.into();
        assert_eq!(recovered, original);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_encoded_bytes_thread_safe() {
        use std::sync::Arc;
        use std::thread;

        let encoded = EncodedBytes::new(&[42]).unwrap();
        let shared = Arc::new(encoded);

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let data = Arc::clone(&shared);
                thread::spawn(move || {
                    assert_eq!(&data[..], &[42]);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    // ========================================================================
    // Security Tests - Malicious Input Handling
    // ========================================================================

    #[test]
    fn security_test_all_continuation_bits() {
        // All bytes have continuation bit set (0xFF = all 1s)
        let malicious = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let result = u64::try_decode_from(&malicious);
        // Should fail due to too many bytes or overflow
        assert!(
            result.is_err(),
            "Should reject all-continuation-bit sequence"
        );
    }

    #[test]
    fn security_test_maximum_length_varint_u64() {
        // u64::MAX encoded as varint (10 bytes)
        let max_encoded = u64::MAX.encode_into();
        assert_eq!(max_encoded.len(), 10);
        let (decoded, remaining) = u64::try_decode_from(&max_encoded).unwrap();
        assert_eq!(decoded, u64::MAX);
        assert!(remaining.is_empty());
    }

    #[test]
    fn security_test_maximum_length_varint_u128() {
        // u128::MAX encoded as varint (19 bytes)
        let max_encoded = u128::MAX.encode_into();
        assert_eq!(max_encoded.len(), 19);
        let (decoded, remaining) = u128::try_decode_from(&max_encoded).unwrap();
        assert_eq!(decoded, u128::MAX);
        assert!(remaining.is_empty());
    }

    #[test]
    fn security_test_single_byte_with_continuation() {
        // Single byte with continuation bit but no follow-up
        let malicious = vec![0x80];
        assert!(u8::try_decode_from(&malicious).is_err());
        assert!(u16::try_decode_from(&malicious).is_err());
        assert!(u32::try_decode_from(&malicious).is_err());
        assert!(u64::try_decode_from(&malicious).is_err());
    }

    #[test]
    fn security_test_multiple_continuation_bytes_truncated() {
        // Multiple continuation bytes that end prematurely
        let malicious = vec![0xFF, 0xFF, 0xFF];
        assert!(u32::try_decode_from(&malicious).is_err());
    }

    #[test]
    fn security_test_encoded_bytes_rejects_invalid() {
        // Test that EncodedBytes rejects various invalid inputs

        // Empty
        assert!(EncodedBytes::try_from(vec![]).is_err());

        // Truncated varint
        assert!(EncodedBytes::try_from(vec![0x80]).is_err());

        // Trailing bytes after valid varint
        let mut with_trailing = vec![42];
        with_trailing.extend_from_slice(&[0xFF, 0xEE]);
        assert!(EncodedBytes::try_from(with_trailing).is_err());

        // All continuation bits
        assert!(EncodedBytes::try_from(vec![0xFF, 0xFF, 0xFF]).is_err());
    }

    #[test]
    fn security_test_buffer_encoding_no_overflow() {
        // Ensure buffer encoding doesn't overflow with many values
        let mut buffer = Vec::new();

        // Encode 1000 values
        for i in 0u16..1000 {
            i.encode_into_buffer(&mut buffer);
        }

        // Should have successfully encoded without panic
        assert!(buffer.len() > 1000); // At least 1 byte per value, some more
        assert!(buffer.len() < 3000); // At most 3 bytes per u16 value
    }

    #[test]
    fn security_test_array_encoding_bounds() {
        // Verify array encoding respects bounds for max values
        let (array, len) = u128::MAX.encode_into_array();
        assert!(len <= 19, "u128::MAX should not exceed 19 bytes");

        // Verify the encoded data is valid
        let (decoded, _) = u128::try_decode_from(&array[..len]).unwrap();
        assert_eq!(decoded, u128::MAX);
    }

    #[test]
    fn security_test_bool_decode_nonzero_values() {
        // Any non-zero value within u8 range should decode as true
        let nonzero_values = vec![
            vec![1],          // 1
            vec![42],         // 42
            vec![127],        // 127
            vec![0xFF, 0x01], // 255 (max u8)
        ];

        for bytes in nonzero_values {
            let (val, _) = bool::try_decode_from(&bytes).unwrap();
            assert!(val, "Non-zero value {:?} should decode as true", bytes);
        }

        // Zero should decode as false
        let (val, _) = bool::try_decode_from(&[0]).unwrap();
        assert!(!val);
    }

    #[test]
    fn security_test_zero_length_slice_rejection() {
        // Zero-length slices should be rejected for all types
        let empty: &[u8] = &[];

        assert!(u8::try_decode_from(empty).is_err());
        assert!(u16::try_decode_from(empty).is_err());
        assert!(u32::try_decode_from(empty).is_err());
        assert!(u64::try_decode_from(empty).is_err());
        assert!(u128::try_decode_from(empty).is_err());
        assert!(usize::try_decode_from(empty).is_err());
        assert!(bool::try_decode_from(empty).is_err());
    }

    #[test]
    fn security_test_repeated_decoding_no_panic() {
        // Decode the same malicious data repeatedly to check for panics
        let malicious = vec![0xFF, 0xFF];

        for _ in 0..100 {
            let _ = u32::try_decode_from(&malicious);
            let _ = u64::try_decode_from(&malicious);
        }
    }

    #[test]
    fn security_test_alternating_bit_patterns() {
        // Test various bit patterns that might trigger edge cases
        let patterns = vec![
            vec![0xAA], // 10101010
            vec![0x55], // 01010101
            vec![0xAA, 0x55],
            vec![0x55, 0xAA],
        ];

        for pattern in patterns {
            // These should either decode successfully or return an error
            // but must not panic
            let _ = u8::try_decode_from(&pattern);
            let _ = u16::try_decode_from(&pattern);
        }
    }

    // ========================================================================
    // Security Tests - Fuzzing-Style Property Tests
    // ========================================================================

    proptest! {
        #[test]
        fn security_prop_decode_random_bytes(bytes in prop::collection::vec(any::<u8>(), 0..50)) {
            // Random bytes should either decode successfully or return an error
            // but must not panic
            let _ = u8::try_decode_from(&bytes);
            let _ = u16::try_decode_from(&bytes);
            let _ = u32::try_decode_from(&bytes);
            let _ = u64::try_decode_from(&bytes);
            let _ = u128::try_decode_from(&bytes);
        }

        #[test]
        fn security_prop_encoded_bytes_validation(bytes in prop::collection::vec(any::<u8>(), 0..100)) {
            // EncodedBytes validation should never panic, only return Ok or Err
            let result = EncodedBytes::try_from(bytes);

            // If it succeeds, the data should be valid
            if let Ok(encoded) = result {
                // Should be able to decode as u128 (covers all smaller types)
                let decode_result = u128::try_decode_from(encoded.as_ref());
                prop_assert!(decode_result.is_ok(), "Validated EncodedBytes should be decodable");
            }
        }

        #[test]
        fn security_prop_buffer_encoding_no_panic(values in prop::collection::vec(any::<u32>(), 0..1000)) {
            // Encoding many values into a buffer should not panic
            let mut buffer = Vec::new();
            for value in values {
                value.encode_into_buffer(&mut buffer);
            }

            // Buffer should have reasonable size
            prop_assert!(buffer.len() < 5000, "Buffer should not grow excessively");
        }

        #[test]
        fn security_prop_array_encoding_never_panics(value: u128) {
            // Array encoding should never panic for any value
            let (_array, len) = value.encode_into_array();

            // Length should be within bounds
            prop_assert!(len > 0);
            prop_assert!(len <= 19);

            // Encode again to get array for decoding
            let (array, len) = value.encode_into_array();
            let (decoded, _) = u128::try_decode_from(&array[..len]).unwrap();
            prop_assert_eq!(decoded, value);
        }

        #[test]
        fn security_prop_decode_never_returns_more_than_input(
            bytes in prop::collection::vec(any::<u8>(), 1..100)
        ) {
            // Decoded remainder should never be larger than input
            if let Ok((_, remaining)) = u64::try_decode_from(&bytes) {
                prop_assert!(remaining.len() <= bytes.len());
            }
        }

        #[test]
        fn security_prop_encode_size_bounded(value: u64) {
            // Encoded size should never exceed maximum varint size for the type
            let encoded = value.encode_into();
            prop_assert!(encoded.len() <= 10, "u64 varint should not exceed 10 bytes");

            let (_array, len) = value.encode_into_array();
            prop_assert!(len <= 10, "u64 array encoding should not exceed 10 bytes");
        }
    }

    // ========================================================================
    // Concurrency Tests - Compile-Time Send + Sync Verification
    // ========================================================================

    /// Verify that primitive types used with traits are Send + Sync
    #[test]
    fn assert_primitives_send_sync() {
        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}

        // All integer types should be Send + Sync
        is_send::<u8>();
        is_sync::<u8>();
        is_send::<u16>();
        is_sync::<u16>();
        is_send::<u32>();
        is_sync::<u32>();
        is_send::<u64>();
        is_sync::<u64>();
        is_send::<u128>();
        is_sync::<u128>();
        is_send::<usize>();
        is_sync::<usize>();
        is_send::<bool>();
        is_sync::<bool>();

        // Vec<u8> should be Send + Sync
        is_send::<Vec<u8>>();
        is_sync::<Vec<u8>>();
    }

    /// Verify that encoded data types are Send + Sync
    #[test]
    fn assert_encoded_types_send_sync() {
        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}

        // EncodedBytes should be Send + Sync
        is_send::<EncodedBytes>();
        is_sync::<EncodedBytes>();

        // Error should be Send + Sync
        is_send::<crate::Error>();
        is_sync::<crate::Error>();
    }

    // ========================================================================
    // Concurrency Tests - Multi-Threaded Encoding
    // ========================================================================

    #[cfg(feature = "std")]
    #[test]
    fn concurrency_test_parallel_encode_into() {
        use std::sync::Arc;
        use std::sync::Mutex;
        use std::thread;

        // Encode values in parallel threads
        let results = Arc::new(Mutex::new(Vec::new()));
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let results = Arc::clone(&results);
                thread::spawn(move || {
                    let value = i as u32 * 100;
                    let encoded = value.encode_into();
                    results.lock().unwrap().push((value, encoded));
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all encodings are correct
        let results = results.lock().unwrap();
        assert_eq!(results.len(), 10);
        for (original, encoded) in results.iter() {
            let (decoded, _) = u32::try_decode_from(encoded).unwrap();
            assert_eq!(*original, decoded);
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn concurrency_test_parallel_encode_into_buffer() {
        use std::thread;

        // Each thread gets its own buffer
        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    let mut buffer = Vec::new();
                    for j in 0u16..100 {
                        (i * 100 + j).encode_into_buffer(&mut buffer);
                    }
                    buffer
                })
            })
            .collect();

        for handle in handles {
            let buffer = handle.join().unwrap();
            assert!(buffer.len() >= 100); // At least 1 byte per value
            assert!(buffer.len() < 300); // At most 3 bytes per u16
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn concurrency_test_parallel_encode_into_array() {
        use std::thread;

        // Array encoding is purely stack-based, perfect for parallelism
        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    let value = i as u64 * 1000;
                    let (array, len) = value.encode_into_array();
                    (value, array, len)
                })
            })
            .collect();

        for handle in handles {
            let (original, array, len) = handle.join().unwrap();
            let (decoded, _) = u64::try_decode_from(&array[..len]).unwrap();
            assert_eq!(original, decoded);
        }
    }

    // ========================================================================
    // Concurrency Tests - Multi-Threaded Decoding
    // ========================================================================

    #[cfg(feature = "std")]
    #[test]
    fn concurrency_test_shared_decode_data() {
        use std::sync::Arc;
        use std::thread;

        // Pre-encode some data
        let mut data = Vec::new();
        for i in 0u32..100 {
            data.extend_from_slice(&i.encode_into());
        }
        let shared_data = Arc::new(data);

        // Multiple threads decode from the same data
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let data = Arc::clone(&shared_data);
                thread::spawn(move || {
                    let mut slice = &data[..];
                    let mut count = 0;
                    while !slice.is_empty() {
                        match u32::try_decode_from(slice) {
                            Ok((_, remaining)) => {
                                count += 1;
                                slice = remaining;
                            }
                            Err(_) => break,
                        }
                    }
                    count
                })
            })
            .collect();

        for handle in handles {
            let count = handle.join().unwrap();
            assert_eq!(count, 100, "All threads should decode 100 values");
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn concurrency_test_parallel_decode_different_data() {
        use std::thread;

        // Each thread decodes different data
        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    // Create encoded data
                    let mut data = Vec::new();
                    for j in 0u16..50 {
                        (i * 50 + j).encode_into_buffer(&mut data);
                    }

                    // Decode it
                    let mut slice = &data[..];
                    let mut decoded_values = Vec::new();
                    while !slice.is_empty() {
                        match u16::try_decode_from(slice) {
                            Ok((value, remaining)) => {
                                decoded_values.push(value);
                                slice = remaining;
                            }
                            Err(_) => break,
                        }
                    }
                    decoded_values
                })
            })
            .collect();

        for (i, handle) in handles.into_iter().enumerate() {
            let values = handle.join().unwrap();
            assert_eq!(values.len(), 50);
            // Verify first and last values
            assert_eq!(values[0], i as u16 * 50);
            assert_eq!(values[49], i as u16 * 50 + 49);
        }
    }

    // ========================================================================
    // Concurrency Tests - Pipeline Processing
    // ========================================================================

    #[cfg(feature = "std")]
    #[test]
    fn concurrency_test_pipeline_encode_decode() {
        use std::sync::mpsc;
        use std::thread;

        let (tx, rx) = mpsc::channel();

        // Encoder thread
        let encoder = thread::spawn(move || {
            for i in 0u32..1000 {
                tx.send(i.encode_into()).unwrap();
            }
        });

        // Decoder thread
        let decoder = thread::spawn(move || {
            let mut sum = 0u64;
            for encoded in rx {
                let (value, _) = u32::try_decode_from(&encoded).unwrap();
                sum += value as u64;
            }
            sum
        });

        encoder.join().unwrap();
        let sum = decoder.join().unwrap();

        // Sum of 0..1000 = 999 * 1000 / 2 = 499500
        assert_eq!(sum, 499_500);
    }

    #[cfg(feature = "std")]
    #[test]
    fn concurrency_test_multi_producer_single_consumer() {
        use std::sync::mpsc;
        use std::thread;

        let (tx, rx) = mpsc::channel();

        // Multiple producer threads
        let producers: Vec<_> = (0..4)
            .map(|thread_id| {
                let tx = tx.clone();
                thread::spawn(move || {
                    for i in 0u16..100 {
                        let value = thread_id * 100 + i;
                        tx.send(value.encode_into()).unwrap();
                    }
                })
            })
            .collect();

        // Drop original sender
        drop(tx);

        // Single consumer thread
        let consumer = thread::spawn(move || {
            let mut count = 0;
            for encoded in rx {
                let (value, _) = u16::try_decode_from(&encoded).unwrap();
                assert!(value < 400);
                count += 1;
            }
            count
        });

        for producer in producers {
            producer.join().unwrap();
        }

        let total = consumer.join().unwrap();
        assert_eq!(total, 400); // 4 threads * 100 values
    }

    // ========================================================================
    // Concurrency Tests - Stress Testing
    // ========================================================================

    #[cfg(feature = "std")]
    #[test]
    fn concurrency_test_stress_parallel_encoding() {
        use std::thread;

        // Stress test with many threads
        let handles: Vec<_> = (0..100)
            .map(|i| {
                thread::spawn(move || {
                    let mut buffer = Vec::new();
                    for j in 0u8..255 {
                        ((i * 255 + j as u32) % u16::MAX as u32).encode_into_buffer(&mut buffer);
                    }
                    buffer.len()
                })
            })
            .collect();

        let mut total_bytes = 0;
        for handle in handles {
            total_bytes += handle.join().unwrap();
        }

        // Should have encoded 100 * 255 = 25500 values
        assert!(total_bytes >= 25500); // At least 1 byte per value
        assert!(total_bytes < 76500); // At most 3 bytes per value
    }

    #[cfg(feature = "std")]
    #[test]
    fn concurrency_test_stress_shared_read() {
        use std::sync::Arc;
        use std::thread;

        // Pre-encode a large dataset
        let mut data = Vec::new();
        for i in 0u32..10000 {
            i.encode_into_buffer(&mut data);
        }
        let shared_data = Arc::new(data);

        // Many threads read the same data
        let handles: Vec<_> = (0..50)
            .map(|_| {
                let data = Arc::clone(&shared_data);
                thread::spawn(move || {
                    let mut slice = &data[..];
                    let mut count = 0;
                    while !slice.is_empty() {
                        if let Ok((_, remaining)) = u32::try_decode_from(slice) {
                            count += 1;
                            slice = remaining;
                        } else {
                            break;
                        }
                    }
                    count
                })
            })
            .collect();

        for handle in handles {
            let count = handle.join().unwrap();
            assert_eq!(count, 10000);
        }
    }

    // ========================================================================
    // Concurrency Tests - Work Stealing Pattern
    // ========================================================================

    #[cfg(feature = "std")]
    #[test]
    fn concurrency_test_work_stealing() {
        use std::sync::Arc;
        use std::sync::Mutex;
        use std::thread;

        // Shared work queue
        let work_queue = Arc::new(Mutex::new((0u32..1000).collect::<Vec<_>>()));
        let results = Arc::new(Mutex::new(Vec::new()));

        // Worker threads steal work from queue
        let workers: Vec<_> = (0..4)
            .map(|_| {
                let queue = Arc::clone(&work_queue);
                let results = Arc::clone(&results);
                thread::spawn(move || {
                    loop {
                        let work_item = {
                            let mut queue = queue.lock().unwrap();
                            queue.pop()
                        };

                        match work_item {
                            Some(value) => {
                                // Encode and decode
                                let encoded = value.encode_into();
                                let (decoded, _) = u32::try_decode_from(&encoded).unwrap();
                                results.lock().unwrap().push(decoded);
                            }
                            None => break,
                        }
                    }
                })
            })
            .collect();

        for worker in workers {
            worker.join().unwrap();
        }

        let results = results.lock().unwrap();
        assert_eq!(results.len(), 1000);
    }
}
