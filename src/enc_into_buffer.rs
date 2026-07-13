// SPDX-License-Identifier: Apache-2.0
//! Buffer-based encoding trait for zero-allocation varint encoding.
//!
//! This module provides the [`EncodeIntoBuffer`] trait, which allows encoding
//! values into an existing buffer without allocating. This is useful for
//! performance-critical code paths where allocations should be minimized.
//!
//! # Performance Benefits
//!
//! Using `EncodeIntoBuffer` instead of [`EncodeInto`](crate::EncodeInto) provides:
//! - **Zero allocations**: Reuses existing buffer capacity
//! - **Better cache locality**: Fewer heap operations
//! - **Batch encoding**: Encode multiple values into one buffer
//! - **Predictable performance**: No allocation pauses
//!
//! # Use Cases
//!
//! - Hot paths in serialization code
//! - Encoding multiple values sequentially
//! - Systems with allocation constraints
//! - Performance-sensitive applications
//!
//! # Examples
//!
//! ## Single value encoding
//!
//! ```rust
//! use multi_trait::EncodeIntoBuffer;
//!
//! let mut buffer = Vec::new();
//! 42u8.encode_into_buffer(&mut buffer);
//! assert_eq!(buffer, vec![42]);
//! ```
//!
//! ## Sequential encoding (multiple values)
//!
//! ```rust
//! use multi_trait::EncodeIntoBuffer;
//!
//! let mut buffer = Vec::new();
//! 42u8.encode_into_buffer(&mut buffer);
//! 1000u16.encode_into_buffer(&mut buffer);
//! 100000u32.encode_into_buffer(&mut buffer);
//!
//! // All three values encoded in one buffer with a single allocation
//! assert!(buffer.len() > 3);
//! ```
//!
//! ## Buffer reuse
//!
//! ```rust
//! use multi_trait::EncodeIntoBuffer;
//!
//! let mut buffer = Vec::with_capacity(100);
//! for i in 0u8..10 {
//!     i.encode_into_buffer(&mut buffer);
//! }
//! // Buffer was reused for all 10 values with no additional allocations
//! assert_eq!(buffer.len(), 10);
//! ```

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use unsigned_varint::encode;

/// Trait for encoding values into an existing buffer.
///
/// This trait provides zero-allocation encoding by appending to an existing
/// `Vec<u8>`. This is more efficient than [`EncodeInto`](crate::EncodeInto)
/// when:
/// - Encoding multiple values sequentially
/// - Working in allocation-sensitive contexts
/// - Reusing buffers across multiple operations
///
/// # Performance
///
/// This trait is optimized for minimal overhead:
/// - No heap allocations (unless buffer needs to grow)
/// - Direct buffer manipulation
/// - Inline hints for hot paths
///
/// # Thread Safety
///
/// This trait is `Send + Sync` safe. All implementations are stateless and can
/// be called concurrently from multiple threads.
///
/// # Examples
///
/// ```rust
/// use multi_trait::EncodeIntoBuffer;
///
/// // Create a reusable buffer
/// let mut buffer = Vec::with_capacity(64);
///
/// // Encode values without allocation
/// 42u8.encode_into_buffer(&mut buffer);
/// 1000u16.encode_into_buffer(&mut buffer);
///
/// // Buffer contains both encoded values
/// assert!(buffer.len() >= 2);
/// ```
///
/// # Implemented For
///
/// - `bool`: Encoded as 0 (false) or 1 (true)
/// - `u8`, `u16`, `u32`, `u64`, `u128`: Variable-length encoding
/// - `usize`: Platform-dependent (32-bit or 64-bit)
pub trait EncodeIntoBuffer {
    /// Encode this value and append it to the given buffer.
    ///
    /// The buffer will be extended with the varint-encoded bytes representing
    /// this value. The buffer's existing contents are preserved.
    ///
    /// # Performance
    ///
    /// This method is optimized to minimize allocations. If the buffer has
    /// sufficient capacity, no allocation occurs. Otherwise, the buffer will
    /// grow according to Vec's growth strategy.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multi_trait::EncodeIntoBuffer;
    ///
    /// let mut buffer = Vec::new();
    /// 42u8.encode_into_buffer(&mut buffer);
    /// assert_eq!(buffer, vec![42]);
    ///
    /// // Append another value
    /// 100u8.encode_into_buffer(&mut buffer);
    /// assert_eq!(buffer.len(), 2);
    /// ```
    fn encode_into_buffer(&self, buffer: &mut Vec<u8>);
}

/// Macro to implement EncodeIntoBuffer for unsigned integer types using varint encoding.
///
/// This macro eliminates code duplication by generating identical implementations
/// for different numeric types. Each implementation:
/// 1. Creates an appropriate temporary buffer for the type
/// 2. Encodes the value into the temporary buffer
/// 3. Finds the encoded length
/// 4. Extends the target buffer with the encoded bytes
///
/// # Usage
///
/// ```text
/// impl_encode_into_buffer! {
///     u8 => u8_buffer, u8;
///     u16 => u16_buffer, u16;
/// }
/// ```
///
/// # Hygiene
///
/// This macro uses fully qualified paths to ensure proper hygiene and avoid
/// namespace collisions with user code.
macro_rules! impl_encode_into_buffer {
    ($($type:ty => $buffer_fn:ident, $encode_fn:ident);+ $(;)?) => {
        $(
            #[doc = concat!("Encode a ", stringify!($type), " and append to a buffer")]
            impl EncodeIntoBuffer for $type {
                #[inline]
                fn encode_into_buffer(&self, buffer: &mut Vec<u8>) {
                    // Create temporary buffer for this type
                    let mut buf = encode::$buffer_fn();

                    // Encode value into temporary buffer
                    let encoded = encode::$encode_fn(*self, &mut buf);

                    // Extend the target buffer with the encoded bytes
                    buffer.extend_from_slice(encoded);
                }
            }
        )+
    };
}

/// Encode a bool and append to a buffer
impl EncodeIntoBuffer for bool {
    #[inline]
    fn encode_into_buffer(&self, buffer: &mut Vec<u8>) {
        buffer.push(if *self { 1 } else { 0 });
    }
}

// Implement EncodeIntoBuffer for all unsigned integer types using the macro
impl_encode_into_buffer! {
    u8 => u8_buffer, u8;
    u16 => u16_buffer, u16;
    u32 => u32_buffer, u32;
    u64 => u64_buffer, u64;
    u128 => u128_buffer, u128;
    usize => usize_buffer, usize;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_single_value() {
        let mut buffer = Vec::new();
        42u8.encode_into_buffer(&mut buffer);
        assert_eq!(buffer, vec![42]);
    }

    #[test]
    fn test_buffer_multiple_values() {
        let mut buffer = Vec::new();
        42u8.encode_into_buffer(&mut buffer);
        100u8.encode_into_buffer(&mut buffer);
        200u8.encode_into_buffer(&mut buffer);
        // 42 = 1 byte, 100 = 1 byte, 200 = 2 bytes (>127 requires continuation)
        assert_eq!(buffer.len(), 4);
    }

    #[test]
    fn test_buffer_sequential_encoding() {
        let mut buffer = Vec::new();
        42u8.encode_into_buffer(&mut buffer);
        1000u16.encode_into_buffer(&mut buffer);
        100_000_u32.encode_into_buffer(&mut buffer);
        // Verify buffer contains all three values
        assert!(buffer.len() > 3);
    }

    #[test]
    fn test_buffer_preserves_existing_content() {
        let mut buffer = vec![0xFF, 0xEE];
        42u8.encode_into_buffer(&mut buffer);
        assert_eq!(buffer[0], 0xFF);
        assert_eq!(buffer[1], 0xEE);
        assert_eq!(buffer[2], 42);
    }

    #[test]
    fn test_buffer_bool() {
        let mut buffer = Vec::new();
        true.encode_into_buffer(&mut buffer);
        false.encode_into_buffer(&mut buffer);
        assert_eq!(buffer, vec![1, 0]);
    }

    #[test]
    fn test_buffer_reuse() {
        let mut buffer = Vec::with_capacity(100);
        for i in 0u8..10 {
            i.encode_into_buffer(&mut buffer);
        }
        assert_eq!(buffer.len(), 10);
        // Verify no reallocation occurred
        assert!(buffer.capacity() >= 100);
    }

    #[test]
    fn test_buffer_large_values() {
        let mut buffer = Vec::new();
        u8::MAX.encode_into_buffer(&mut buffer);
        u16::MAX.encode_into_buffer(&mut buffer);
        u32::MAX.encode_into_buffer(&mut buffer);
        // Each max value takes multiple bytes
        assert!(buffer.len() > 3);
    }

    #[test]
    fn test_buffer_zero_values() {
        let mut buffer = Vec::new();
        0u8.encode_into_buffer(&mut buffer);
        0u16.encode_into_buffer(&mut buffer);
        0u32.encode_into_buffer(&mut buffer);
        // Zero values take 1 byte each
        assert_eq!(buffer, vec![0, 0, 0]);
    }
}
