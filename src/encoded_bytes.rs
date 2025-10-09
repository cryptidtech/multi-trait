// SPDX-License-Identifier: Apache-2.0
//! Validated newtype for varint-encoded byte sequences.
//!
//! This module provides the [`EncodedBytes`] type, a validated wrapper around
//! `Vec<u8>` that guarantees the bytes represent a valid, complete varint-encoded
//! value. This follows the "parse, don't validate" principle by making invalid
//! states unrepresentable at the type level.

#[cfg(not(feature = "std"))]
use alloc::{format, vec::Vec};

use crate::error::Error;
use core::ops::Deref;
use unsigned_varint::decode;

/// A validated wrapper for varint-encoded bytes.
///
/// This newtype ensures that the contained bytes form a valid, complete
/// varint-encoded value. The validation is performed at construction time,
/// making it impossible to create an `EncodedBytes` instance with invalid data.
///
/// # Type Safety Benefits
///
/// Using `EncodedBytes` provides several advantages:
///
/// - **Validation at the boundary**: Invalid data is rejected at construction
/// - **Type-level guarantees**: If you have an `EncodedBytes`, it's valid
/// - **Clear API contracts**: Functions accepting `EncodedBytes` don't need validation
/// - **Zero-cost abstraction**: No runtime overhead after construction
///
/// # Thread Safety
///
/// `EncodedBytes` is both `Send` and `Sync`, making it safe to share across
/// thread boundaries. The contained data is immutable after validation.
///
/// # Examples
///
/// ## Creating from valid data
///
/// ```rust
/// use multitrait::EncodedBytes;
///
/// // Valid varint encoding of 42
/// let bytes = vec![42u8];
/// let encoded = EncodedBytes::try_from(bytes).unwrap();
/// assert_eq!(encoded.as_ref(), &[42]);
/// ```
///
/// ## Validation catches invalid data
///
/// ```rust
/// use multitrait::EncodedBytes;
///
/// // Invalid: continuation bit set but no following byte
/// let invalid = vec![0x80];
/// let result = EncodedBytes::try_from(invalid);
/// assert!(result.is_err());
/// ```
///
/// ## Zero-cost unwrapping
///
/// ```rust
/// use multitrait::EncodedBytes;
///
/// let bytes = vec![42u8];
/// let encoded = EncodedBytes::try_from(bytes).unwrap();
///
/// // Convert back to Vec<u8> with no allocation
/// let original: Vec<u8> = encoded.into();
/// assert_eq!(original, vec![42]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EncodedBytes(Vec<u8>);

impl EncodedBytes {
    /// Create a new `EncodedBytes` from a byte slice, validating the data.
    ///
    /// This performs the same validation as `TryFrom<Vec<u8>>` but works
    /// with slices, requiring an allocation.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes don't form a valid, complete varint.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multitrait::EncodedBytes;
    ///
    /// let encoded = EncodedBytes::new(&[42]).unwrap();
    /// assert_eq!(encoded.as_ref(), &[42]);
    /// ```
    pub fn new(bytes: &[u8]) -> Result<Self, Error> {
        Self::try_from(bytes.to_vec())
    }

    /// Returns the length of the encoded data in bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multitrait::EncodedBytes;
    ///
    /// let encoded = EncodedBytes::new(&[42]).unwrap();
    /// assert_eq!(encoded.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the encoded data is empty.
    ///
    /// Note: Empty `EncodedBytes` cannot be constructed through normal means,
    /// as empty bytes don't represent a valid varint. This method exists for
    /// API completeness.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multitrait::EncodedBytes;
    ///
    /// let encoded = EncodedBytes::new(&[42]).unwrap();
    /// assert!(!encoded.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns a reference to the underlying bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multitrait::EncodedBytes;
    ///
    /// let encoded = EncodedBytes::new(&[42]).unwrap();
    /// assert_eq!(encoded.as_bytes(), &[42]);
    /// ```
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Consumes the `EncodedBytes` and returns the underlying `Vec<u8>`.
    ///
    /// This is a zero-cost operation that transfers ownership without cloning.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multitrait::EncodedBytes;
    ///
    /// let encoded = EncodedBytes::new(&[42]).unwrap();
    /// let bytes = encoded.into_vec();
    /// assert_eq!(bytes, vec![42]);
    /// ```
    #[inline]
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl TryFrom<Vec<u8>> for EncodedBytes {
    type Error = Error;

    /// Attempt to create `EncodedBytes` from a `Vec<u8>`, validating the data.
    ///
    /// This validates that the bytes form a complete, valid varint encoding.
    /// The validation checks:
    /// 1. The bytes are not empty
    /// 2. The varint can be successfully decoded
    /// 3. All bytes are consumed (no trailing data)
    ///
    /// # Errors
    ///
    /// Returns [`Error::InsufficientData`] if the bytes are empty or incomplete.
    /// Returns [`Error::UnsignedVarintDecode`] if the varint encoding is invalid.
    /// Returns [`Error::InvalidEncoding`] if there are trailing bytes after a
    /// valid varint.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multitrait::EncodedBytes;
    ///
    /// // Valid single-byte encoding
    /// let valid = vec![42];
    /// assert!(EncodedBytes::try_from(valid).is_ok());
    ///
    /// // Invalid: continuation bit without following byte
    /// let invalid = vec![0x80];
    /// assert!(EncodedBytes::try_from(invalid).is_err());
    /// ```
    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        // Check for empty input
        if bytes.is_empty() {
            return Err(Error::InsufficientData {
                expected: 1,
                actual: 0,
            });
        }

        // Validate that bytes form a complete varint by attempting decode
        // We use u128 as it can represent any varint value
        let remaining = match decode::u128(&bytes) {
            Ok((_, remaining)) => remaining,
            Err(source) => {
                #[cfg(feature = "std")]
                { return Err(Error::UnsignedVarintDecode { source }); }
                #[cfg(not(feature = "std"))]
                { return Err(Error::UnsignedVarintDecode { message: format!("{:?}", source) }); }
            }
        };

        // Ensure no trailing bytes
        if !remaining.is_empty() {
            return Err(Error::InvalidEncoding {
                reason: format!(
                    "trailing bytes after valid varint: {} bytes remaining",
                    remaining.len()
                ),
            });
        }

        Ok(EncodedBytes(bytes))
    }
}

impl From<EncodedBytes> for Vec<u8> {
    /// Convert `EncodedBytes` back into a `Vec<u8>`.
    ///
    /// This is a zero-cost operation that consumes the `EncodedBytes`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multitrait::EncodedBytes;
    ///
    /// let encoded = EncodedBytes::new(&[42]).unwrap();
    /// let bytes: Vec<u8> = encoded.into();
    /// assert_eq!(bytes, vec![42]);
    /// ```
    #[inline]
    fn from(encoded: EncodedBytes) -> Self {
        encoded.0
    }
}

impl AsRef<[u8]> for EncodedBytes {
    /// Returns a reference to the encoded bytes as a slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multitrait::EncodedBytes;
    ///
    /// let encoded = EncodedBytes::new(&[42]).unwrap();
    /// let slice: &[u8] = encoded.as_ref();
    /// assert_eq!(slice, &[42]);
    /// ```
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for EncodedBytes {
    type Target = [u8];

    /// Dereferences to the underlying byte slice.
    ///
    /// This allows `EncodedBytes` to be used anywhere a `&[u8]` is expected.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multitrait::EncodedBytes;
    ///
    /// let encoded = EncodedBytes::new(&[42]).unwrap();
    /// assert_eq!(encoded[0], 42); // Deref enables indexing
    /// ```
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Explicitly document Send + Sync bounds
//
// SAFETY: EncodedBytes is Send + Sync because:
// 1. It contains only a Vec<u8>, which is Send + Sync
// 2. The data is immutable after validation
// 3. No interior mutability is used
// 4. All validation happens at construction time
unsafe impl Send for EncodedBytes {}
unsafe impl Sync for EncodedBytes {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_single_byte() {
        let bytes = vec![42];
        let encoded = EncodedBytes::try_from(bytes).unwrap();
        assert_eq!(encoded.as_ref(), &[42]);
        assert_eq!(encoded.len(), 1);
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_valid_multi_byte() {
        // Varint encoding of 128 requires 2 bytes
        let bytes = vec![0x80, 0x01];
        let encoded = EncodedBytes::try_from(bytes).unwrap();
        assert_eq!(encoded.len(), 2);
    }

    #[test]
    fn test_empty_bytes_rejected() {
        let empty: Vec<u8> = vec![];
        let result = EncodedBytes::try_from(empty);
        assert!(result.is_err());
        if let Err(Error::InsufficientData { expected, actual }) = result {
            assert_eq!(expected, 1);
            assert_eq!(actual, 0);
        } else {
            panic!("Expected InsufficientData error");
        }
    }

    #[test]
    fn test_truncated_varint_rejected() {
        // Continuation bit set but no following byte
        let truncated = vec![0x80];
        let result = EncodedBytes::try_from(truncated);
        assert!(result.is_err());
    }

    #[test]
    fn test_trailing_bytes_rejected() {
        // Valid varint followed by extra bytes
        let mut bytes = vec![42];
        bytes.extend_from_slice(&[0xFF, 0xEE]);
        let result = EncodedBytes::try_from(bytes);
        assert!(result.is_err());
        if let Err(Error::InvalidEncoding { reason }) = result {
            assert!(reason.contains("trailing bytes"));
        } else {
            panic!("Expected InvalidEncoding error");
        }
    }

    #[test]
    fn test_into_vec() {
        // Use proper varint encoding
        let original = vec![0x80, 0x01]; // Varint encoding of 128
        let encoded = EncodedBytes::try_from(original.clone()).unwrap();
        let recovered: Vec<u8> = encoded.into();
        assert_eq!(recovered, original);
    }

    #[test]
    fn test_deref() {
        let encoded = EncodedBytes::new(&[42]).unwrap();
        assert_eq!(encoded[0], 42); // Tests Deref
    }

    #[test]
    fn test_clone() {
        let encoded = EncodedBytes::new(&[42]).unwrap();
        let cloned = encoded.clone();
        assert_eq!(encoded, cloned);
    }

    #[test]
    fn test_debug() {
        let encoded = EncodedBytes::new(&[42]).unwrap();
        let debug_str = format!("{:?}", encoded);
        assert!(debug_str.contains("EncodedBytes"));
    }

    // Compile-time verification of Send + Sync
    #[allow(dead_code)]
    fn assert_send_sync() {
        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}
        is_send::<EncodedBytes>();
        is_sync::<EncodedBytes>();
    }
}
