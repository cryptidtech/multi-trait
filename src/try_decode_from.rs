// SPDX-License-Identifier: Apache-2.0
use crate::Error;
use unsigned_varint::decode;

/// Trait for fallibly decoding values from byte slices.
///
/// This trait provides zero-copy parsing of varint-encoded values from byte slices,
/// returning both the decoded value and the remaining unconsumed bytes. This design
/// enables efficient sequential parsing of multiple values from a single buffer.
///
/// # Lifetime Parameter
///
/// The `'a` lifetime parameter ties the returned slice reference to the input buffer,
/// preventing dangling references and enabling zero-copy parsing.
///
/// # Error Handling
///
/// Decoding can fail for several reasons:
/// - Insufficient bytes in the input
/// - Invalid varint encoding
/// - Truncated data
///
/// Errors are returned as structured [`Error`] types with proper
/// source chains for debugging.
///
/// # Performance
///
/// Decoding is highly optimized:
/// - Zero allocations (returns slice references)
/// - No data copying
/// - Constant-time validation
///
/// # Thread Safety
///
/// This trait is `Send + Sync` safe. Implementations are stateless and can be
/// called concurrently from multiple threads.
///
/// # Examples
///
/// ## Basic Decoding
///
/// ```rust
/// use multi_trait::TryDecodeFrom;
///
/// // Decode a single value
/// let bytes = vec![42];
/// let (value, remaining) = u8::try_decode_from(&bytes).unwrap();
/// assert_eq!(value, 42);
/// assert!(remaining.is_empty());
/// ```
///
/// ## Sequential Decoding
///
/// ```rust
/// use multi_trait::TryDecodeFrom;
///
/// // Decode multiple values from one buffer
/// let bytes = vec![0x01, 0x02, 0x03];
/// let (first, rest) = u8::try_decode_from(&bytes).unwrap();
/// let (second, rest) = u8::try_decode_from(rest).unwrap();
/// let (third, rest) = u8::try_decode_from(rest).unwrap();
///
/// assert_eq!(first, 1);
/// assert_eq!(second, 2);
/// assert_eq!(third, 3);
/// assert!(rest.is_empty());
/// ```
///
/// ## Error Handling
///
/// ```rust
/// use multi_trait::{TryDecodeFrom, Error};
///
/// // Handle decode errors
/// let empty: &[u8] = &[];
/// match u32::try_decode_from(empty) {
///     Ok((value, _)) => println!("Decoded: {}", value),
///     Err(Error::UnsignedVarintDecode { .. }) => {
///         // Expected for empty input
///     }
///     Err(e) => panic!("Unexpected error: {}", e),
/// }
/// ```
///
/// # Implemented For
///
/// - `bool`: Decodes 0 as false, non-zero as true
/// - `u8`, `u16`, `u32`, `u64`, `u128`: Variable-length decoding
/// - `usize`: Platform-dependent (32-bit or 64-bit)
pub trait TryDecodeFrom<'a>: Sized {
    /// The error type returned on decoding failure.
    ///
    /// For built-in implementations, this is [`Error`].
    type Error;

    /// Try to decode a value from a byte slice.
    ///
    /// On success, returns a tuple containing:
    /// 1. The decoded value
    /// 2. A slice reference to the remaining unconsumed bytes
    ///
    /// The remaining bytes slice shares the same lifetime as the input,
    /// enabling zero-copy sequential parsing.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The input slice is too short
    /// - The varint encoding is invalid
    /// - The encoded value is truncated
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multi_trait::TryDecodeFrom;
    ///
    /// let bytes = vec![0xFF, 0x01]; // Varint encoding of 255
    /// let (value, remaining) = u8::try_decode_from(&bytes).unwrap();
    /// assert_eq!(value, 255);
    /// assert!(remaining.is_empty());
    /// ```
    fn try_decode_from(bytes: &'a [u8]) -> Result<(Self, &'a [u8]), Self::Error>;
}

/// Macro to implement TryDecodeFrom for unsigned integer types using varint decoding.
///
/// This macro eliminates code duplication by generating identical implementations
/// for different numeric types. Each implementation:
/// 1. Calls the appropriate decode function from unsigned_varint
/// 2. Maps any decode error to a properly structured Error with source chain
/// 3. Returns the decoded value and remaining bytes
///
/// # Usage
///
/// ```text
/// impl_try_decode_from! {
///     u8 => u8;
///     u16 => u16;
/// }
/// ```
///
/// # Error Handling
///
/// The macro properly constructs errors with source chains for debugging,
/// following Rust error handling best practices.
///
/// # Hygiene
///
/// This macro uses fully qualified paths to ensure proper hygiene and avoid
/// namespace collisions with user code.
macro_rules! impl_try_decode_from {
    ($($type:ty => $decode_fn:ident);+ $(;)?) => {
        $(
            #[doc = concat!("Try to decode a varuint encoded ", stringify!($type))]
            impl<'a> TryDecodeFrom<'a> for $type {
                type Error = Error;

                fn try_decode_from(bytes: &'a [u8]) -> Result<(Self, &'a [u8]), Self::Error> {
                    decode::$decode_fn(bytes)
                        .map_err(|source| {
                            #[cfg(feature = "std")]
                            { Self::Error::UnsignedVarintDecode { source } }
                            #[cfg(not(feature = "std"))]
                            { Self::Error::UnsignedVarintDecode { message: alloc::format!("{:?}", source) } }
                        })
                }
            }
        )+
    };
}

/// Try to decode a varuint encoded bool
impl<'a> TryDecodeFrom<'a> for bool {
    type Error = Error;

    fn try_decode_from(bytes: &'a [u8]) -> Result<(Self, &'a [u8]), Self::Error> {
        let (v, ptr) = decode::u8(bytes).map_err(|source| {
            #[cfg(feature = "std")]
            {
                Self::Error::UnsignedVarintDecode { source }
            }
            #[cfg(not(feature = "std"))]
            {
                Self::Error::UnsignedVarintDecode {
                    message: alloc::format!("{:?}", source),
                }
            }
        })?;
        Ok(((v != 0), ptr))
    }
}

// Implement TryDecodeFrom for all unsigned integer types using the macro
impl_try_decode_from! {
    u8 => u8;
    u16 => u16;
    u32 => u32;
    u64 => u64;
    u128 => u128;
    usize => usize;
}

/// Decode a fixed-length byte array (reads N bytes; used for BLS share identifiers).
impl<'a, const N: usize> TryDecodeFrom<'a> for [u8; N] {
    type Error = Error;

    fn try_decode_from(bytes: &'a [u8]) -> Result<([u8; N], &'a [u8]), Self::Error> {
        if bytes.len() < N {
            return Err(Error::InsufficientData {
                expected: N,
                actual: bytes.len(),
            });
        }
        let (head, rest) = bytes.split_at(N);
        let arr = <[u8; N]>::try_from(head).map_err(|_| Error::InsufficientData {
            expected: N,
            actual: bytes.len(),
        })?;
        Ok((arr, rest))
    }
}
