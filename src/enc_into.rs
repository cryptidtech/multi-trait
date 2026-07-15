// SPDX-License-Identifier: Apache-2.0
#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

use unsigned_varint::encode;

/// Trait for encoding values into compact varint byte representation.
///
/// This trait provides efficient encoding of numeric values into unsigned varint
/// format, which uses fewer bytes for smaller values. The encoding is deterministic
/// and reversible via [`TryDecodeFrom`](crate::TryDecodeFrom).
///
/// # Varint Format
///
/// The varint (variable-length integer) format uses the most significant bit (MSB)
/// of each byte as a continuation bit:
/// - If MSB is 1, more bytes follow
/// - If MSB is 0, this is the last byte
///
/// This allows small values to use fewer bytes:
/// - Values 0-127: 1 byte
/// - Values 128-16,383: 2 bytes
/// - And so on...
///
/// # Performance
///
/// Encoding is optimized for performance:
/// - Single heap allocation per call
/// - O(1) length calculation
/// - No byte-by-byte copying
///
/// # Thread Safety
///
/// This trait is `Send + Sync` safe. All implementations are stateless and can
/// be called concurrently from multiple threads.
///
/// # Examples
///
/// ```rust
/// use multi_trait::EncodeInto;
///
/// // Small values use minimal space
/// let small = 42u8;
/// let encoded = small.encode_into();
/// assert_eq!(encoded.len(), 1);
///
/// // Larger values use more bytes as needed
/// let large = 256u16;
/// let encoded = large.encode_into();
/// assert!(encoded.len() > 1);
///
/// // Boolean encoding
/// assert_eq!(true.encode_into(), vec![1]);
/// assert_eq!(false.encode_into(), vec![0]);
/// ```
///
/// # Implemented For
///
/// - `bool`: Encoded as 0 (false) or 1 (true)
/// - `u8`, `u16`, `u32`, `u64`, `u128`: Variable-length encoding
/// - `usize`: Platform-dependent (32-bit or 64-bit)
pub trait EncodeInto {
    /// Encode this value into a compact varint `Vec<u8>`.
    ///
    /// This method allocates a new `Vec` containing the encoded bytes.
    /// The resulting vector's length depends on the value's magnitude.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the varint-encoded representation of this value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multi_trait::EncodeInto;
    ///
    /// let value = 300u16;
    /// let bytes = value.encode_into();
    /// // The exact bytes depend on varint encoding rules
    /// assert!(!bytes.is_empty());
    /// ```
    fn encode_into(&self) -> Vec<u8>;
}

/// Macro to implement `EncodeInto` for unsigned integer types using varint encoding.
///
/// This macro eliminates code duplication by generating identical implementations
/// for different numeric types. Each implementation:
/// 1. Creates an appropriate buffer for the type
/// 2. Encodes the value into the buffer
/// 3. Finds the encoded length by locating the last byte marker
/// 4. Returns a Vec containing only the encoded bytes
///
/// # Usage
///
/// ```text
/// impl_encode_into! {
///     u8 => u8_buffer, u8;
///     u16 => u16_buffer, u16;
/// }
/// ```
///
/// # Hygiene
///
/// This macro uses fully qualified paths to ensure proper hygiene and avoid
/// namespace collisions with user code.
macro_rules! impl_encode_into {
    ($($type:ty => $buffer_fn:ident, $encode_fn:ident);+ $(;)?) => {
        $(
            #[doc = concat!("Encode a ", stringify!($type), " into a compact varuint `Vec<u8>`")]
            impl EncodeInto for $type {
                #[inline]
                fn encode_into(&self) -> Vec<u8> {
                    // Create appropriate buffer for this type
                    let mut buf = encode::$buffer_fn();

                    // Encode value into buffer
                    let encoded = encode::$encode_fn(*self, &mut buf);

                    // Single allocation: slice and convert to Vec
                    encoded.to_vec()
                }
            }
        )+
    };
}

/// Encode a bool into a compact varuint `Vec<u8>`
impl EncodeInto for bool {
    #[inline]
    fn encode_into(&self) -> Vec<u8> {
        if *self { vec![1u8] } else { vec![0u8] }
    }
}

// Implement EncodeInto for all unsigned integer types using the macro
impl_encode_into! {
    u8 => u8_buffer, u8;
    u16 => u16_buffer, u16;
    u32 => u32_buffer, u32;
    u64 => u64_buffer, u64;
    u128 => u128_buffer, u128;
    usize => usize_buffer, usize;
}

/// Encode a fixed-length byte array as raw bytes (used for BLS share identifiers).
impl<const N: usize> EncodeInto for [u8; N] {
    #[inline]
    fn encode_into(&self) -> Vec<u8> {
        self.as_slice().to_vec()
    }
}
