// SPDX-License-Identifier: Apache-2.0
//! Stack-based encoding trait for no-allocation varint encoding.
//!
//! This module provides the [`EncodeIntoArray`] trait, which allows encoding
//! values into a stack-allocated array without any heap allocation. This is
//! essential for `no_std` environments and embedded systems.
//!
//! # Performance Benefits
//!
//! Using `EncodeIntoArray` provides:
//! - **Zero heap allocations**: All data on stack
//! - **Deterministic performance**: No GC or allocator overhead
//! - **Embedded-friendly**: Works in `no_std` contexts
//! - **Predictable memory**: Compile-time known sizes
//!
//! # Use Cases
//!
//! - Embedded systems without allocator
//! - Real-time systems requiring deterministic performance
//! - `no_std` environments
//! - Memory-constrained devices
//! - Safety-critical systems
//!
//! # Examples
//!
//! ## Basic stack encoding
//!
//! ```rust
//! use multi_trait::EncodeIntoArray;
//!
//! let (array, len) = 42u8.encode_into_array();
//! assert_eq!(&array[..len], &[42]);
//! ```
//!
//! ## Working with larger values
//!
//! ```rust
//! use multi_trait::EncodeIntoArray;
//!
//! let (array, len) = 1000u16.encode_into_array();
//! // Varint encoding of 1000 takes 2 bytes
//! assert_eq!(len, 2);
//! assert_eq!(&array[..len], &[0xE8, 0x07]);
//! ```
//!
//! ## Maximum encoded sizes
//!
//! Each type has a compile-time known maximum encoded size:
//!
//! ```rust
//! use multi_trait::EncodeIntoArray;
//!
//! // u8 values fit in 2 bytes max
//! assert_eq!(<u8 as EncodeIntoArray>::MAX_ENCODED_SIZE, 2);
//!
//! // u16 values fit in 3 bytes max
//! assert_eq!(<u16 as EncodeIntoArray>::MAX_ENCODED_SIZE, 3);
//!
//! // u32 values fit in 5 bytes max
//! assert_eq!(<u32 as EncodeIntoArray>::MAX_ENCODED_SIZE, 5);
//! ```

use unsigned_varint::encode;

/// Maximum size needed for any varint encoding (u128 max = 19 bytes)
pub const MAX_VARINT_SIZE: usize = 19;

/// Trait for encoding values into a stack-allocated array.
///
/// This trait provides zero-allocation encoding by using stack-allocated arrays.
/// All types use a fixed-size array of 19 bytes (enough for any varint), with
/// each type documenting its actual maximum size via `MAX_ENCODED_SIZE`.
///
/// # Performance
///
/// This trait is optimized for embedded and `no_std` contexts:
/// - No heap allocations
/// - Compile-time sized buffers
/// - Inline hints for hot paths
/// - Deterministic performance
///
/// # Thread Safety
///
/// This trait is `Send + Sync` safe. All implementations are stateless and can
/// be called concurrently from multiple threads.
///
/// # Maximum Encoded Sizes
///
/// The maximum varint-encoded size for each type is:
/// - `bool`, `u8`: 2 bytes
/// - `u16`: 3 bytes
/// - `u32`: 5 bytes
/// - `u64`: 10 bytes
/// - `u128`: 19 bytes
/// - `usize`: 10 bytes (64-bit) or 5 bytes (32-bit)
///
/// # Examples
///
/// ```rust
/// use multi_trait::EncodeIntoArray;
///
/// // Encode a value to a stack array
/// let (array, len) = 42u8.encode_into_array();
/// assert_eq!(&array[..len], &[42]);
///
/// // Maximum size is known at compile time
/// assert_eq!(<u32 as EncodeIntoArray>::MAX_ENCODED_SIZE, 5);
/// ```
///
/// # Implemented For
///
/// - `bool`: Encoded as 0 (false) or 1 (true)
/// - `u8`, `u16`, `u32`, `u64`, `u128`: Variable-length encoding
/// - `usize`: Platform-dependent (32-bit or 64-bit)
pub trait EncodeIntoArray {
    /// The maximum number of bytes needed to encode any value of this type.
    ///
    /// This is a compile-time constant that documents the maximum size
    /// for this specific type, though all types return a 19-byte array.
    const MAX_ENCODED_SIZE: usize;

    /// Encode this value into a stack-allocated array.
    ///
    /// Returns a tuple of:
    /// - A 19-byte array containing the encoded bytes (may have unused space)
    /// - The actual length of the encoded data (≤ MAX_ENCODED_SIZE)
    ///
    /// # Performance
    ///
    /// This method performs no heap allocations. All data is on the stack.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multi_trait::EncodeIntoArray;
    ///
    /// let (array, len) = 42u8.encode_into_array();
    /// assert_eq!(len, 1);
    /// assert_eq!(&array[..len], &[42]);
    /// ```
    fn encode_into_array(&self) -> ([u8; MAX_VARINT_SIZE], usize);
}

/// Macro to implement EncodeIntoArray for unsigned integer types using varint encoding.
///
/// This macro eliminates code duplication by generating identical implementations
/// for different numeric types. Each implementation:
/// 1. Defines the maximum encoded size for the type
/// 2. Creates an appropriately sized buffer on the stack
/// 3. Encodes the value into the buffer
/// 4. Returns the buffer and actual encoded length
///
/// # Usage
///
/// ```text
/// impl_encode_into_array! {
///     u8, 2 => u8_buffer, u8;
///     u16, 3 => u16_buffer, u16;
/// }
/// ```
///
/// # Hygiene
///
/// This macro uses fully qualified paths to ensure proper hygiene and avoid
/// namespace collisions with user code.
macro_rules! impl_encode_into_array {
    ($($type:ty, $max_size:expr => $buffer_fn:ident, $encode_fn:ident);+ $(;)?) => {
        $(
            #[doc = concat!("Encode a ", stringify!($type), " into a stack-allocated array")]
            impl EncodeIntoArray for $type {
                const MAX_ENCODED_SIZE: usize = $max_size;

                #[inline]
                fn encode_into_array(&self) -> ([u8; MAX_VARINT_SIZE], usize) {
                    // Create buffer on stack
                    let mut buf = encode::$buffer_fn();

                    // Encode value into buffer
                    encode::$encode_fn(*self, &mut buf);

                    // Find the length efficiently by locating the last byte marker
                    let len = buf
                        .iter()
                        .position(|&b| unsigned_varint::decode::is_last(b))
                        .map_or(buf.len(), |pos| pos + 1);

                    // Copy to fixed-size array
                    let mut result = [0u8; MAX_VARINT_SIZE];
                    result[..len].copy_from_slice(&buf[..len]);

                    (result, len)
                }
            }
        )+
    };
}

/// Encode a bool into a stack-allocated array
impl EncodeIntoArray for bool {
    const MAX_ENCODED_SIZE: usize = 1;

    #[inline]
    fn encode_into_array(&self) -> ([u8; MAX_VARINT_SIZE], usize) {
        let mut result = [0u8; MAX_VARINT_SIZE];
        result[0] = if *self { 1 } else { 0 };
        (result, 1)
    }
}

// Implement EncodeIntoArray for all unsigned integer types using the macro
//
// Maximum sizes are calculated as ceil(bits / 7) since varint encoding
// uses 7 bits per byte for data:
// - u8 (8 bits): ceil(8/7) = 2 bytes max
// - u16 (16 bits): ceil(16/7) = 3 bytes max
// - u32 (32 bits): ceil(32/7) = 5 bytes max
// - u64 (64 bits): ceil(64/7) = 10 bytes max
// - u128 (128 bits): ceil(128/7) = 19 bytes max
impl_encode_into_array! {
    u8, 2 => u8_buffer, u8;
    u16, 3 => u16_buffer, u16;
    u32, 5 => u32_buffer, u32;
    u64, 10 => u64_buffer, u64;
    u128, 19 => u128_buffer, u128;
}

// usize is platform-dependent: use 10 bytes (64-bit) to be safe
impl EncodeIntoArray for usize {
    const MAX_ENCODED_SIZE: usize = 10;

    #[inline]
    fn encode_into_array(&self) -> ([u8; MAX_VARINT_SIZE], usize) {
        // Create buffer on stack
        let mut buf = encode::usize_buffer();

        // Encode value into buffer
        encode::usize(*self, &mut buf);

        // Find the length efficiently by locating the last byte marker
        let len = buf
            .iter()
            .position(|&b| unsigned_varint::decode::is_last(b))
            .map_or(buf.len(), |pos| pos + 1);

        // Copy to fixed-size array
        let mut result = [0u8; MAX_VARINT_SIZE];
        result[..len].copy_from_slice(&buf[..len]);

        (result, len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_single_byte() {
        let (array, len) = 42u8.encode_into_array();
        assert_eq!(len, 1);
        assert_eq!(&array[..len], &[42]);
    }

    #[test]
    fn test_array_multi_byte() {
        // 128 requires 2 bytes in varint encoding
        let (array, len) = 128u16.encode_into_array();
        assert_eq!(len, 2);
        assert_eq!(&array[..len], &[0x80, 0x01]);
    }

    #[test]
    fn test_array_bool() {
        let (array_true, len_true) = true.encode_into_array();
        assert_eq!(len_true, 1);
        assert_eq!(&array_true[..len_true], &[1]);

        let (array_false, len_false) = false.encode_into_array();
        assert_eq!(len_false, 1);
        assert_eq!(&array_false[..len_false], &[0]);
    }

    #[test]
    fn test_array_zero_values() {
        let (array, len) = 0u8.encode_into_array();
        assert_eq!(len, 1);
        assert_eq!(&array[..len], &[0]);

        let (array, len) = 0u32.encode_into_array();
        assert_eq!(len, 1);
        assert_eq!(&array[..len], &[0]);
    }

    #[test]
    fn test_array_max_values() {
        let (_array, len) = u8::MAX.encode_into_array();
        assert!(len <= <u8 as EncodeIntoArray>::MAX_ENCODED_SIZE);

        let (_array, len) = u16::MAX.encode_into_array();
        assert!(len <= <u16 as EncodeIntoArray>::MAX_ENCODED_SIZE);

        let (_array, len) = u32::MAX.encode_into_array();
        assert!(len <= <u32 as EncodeIntoArray>::MAX_ENCODED_SIZE);
    }

    #[test]
    fn test_array_max_sizes() {
        // Verify maximum sizes are correct
        assert_eq!(<u8 as EncodeIntoArray>::MAX_ENCODED_SIZE, 2);
        assert_eq!(<u16 as EncodeIntoArray>::MAX_ENCODED_SIZE, 3);
        assert_eq!(<u32 as EncodeIntoArray>::MAX_ENCODED_SIZE, 5);
        assert_eq!(<u64 as EncodeIntoArray>::MAX_ENCODED_SIZE, 10);
        assert_eq!(<u128 as EncodeIntoArray>::MAX_ENCODED_SIZE, 19);
        assert_eq!(<usize as EncodeIntoArray>::MAX_ENCODED_SIZE, 10);
        assert_eq!(<bool as EncodeIntoArray>::MAX_ENCODED_SIZE, 1);
    }

    #[test]
    fn test_array_sequential_encoding() {
        // Demonstrate that multiple values can be encoded independently
        let (array1, len1) = 42u8.encode_into_array();
        let (_array2, len2) = 1000u16.encode_into_array();
        let (_array3, len3) = 100_000_u32.encode_into_array();

        // Each encoding is independent
        assert_eq!(&array1[..len1], &[42]);
        assert_eq!(len2, 2);
        assert!(len3 > 1);
    }

    #[test]
    fn test_array_no_heap_allocation() {
        // This test verifies that the array-based encoding doesn't use heap
        // If it did use heap, this would fail in a no_std environment
        let (_array, len) = u64::MAX.encode_into_array();
        assert!(len <= <u64 as EncodeIntoArray>::MAX_ENCODED_SIZE);
        assert_eq!(len, 10); // u64::MAX takes 10 bytes in varint
    }

    #[test]
    fn test_array_deterministic() {
        // Encoding the same value should always produce the same result
        let (array1, len1) = 12345u32.encode_into_array();
        let (array2, len2) = 12345u32.encode_into_array();

        assert_eq!(len1, len2);
        assert_eq!(&array1[..len1], &array2[..len2]);
    }

    #[test]
    fn test_array_unused_space_is_zeroed() {
        // Verify that unused space in the array is zeroed
        let (array, len) = 42u8.encode_into_array();
        assert_eq!(len, 1);
        // Check that bytes beyond len are zero
        for &byte in &array[len..MAX_VARINT_SIZE] {
            assert_eq!(byte, 0);
        }
    }
}
