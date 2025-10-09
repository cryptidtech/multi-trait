// SPDX-License-Identifier: Apache-2.0
//! Error handling example
//!
//! This example demonstrates proper error handling patterns when using
//! the multitrait crate, including:
//! - Handling decode errors
//! - Inspecting error sources
//! - Validating data with EncodedBytes
//! - Recovering from errors

use multitrait::{EncodedBytes, Error, TryDecodeFrom};

fn main() {
    println!("=== Multitrait Error Handling Example ===\n");

    // Example 1: Handling empty input
    handle_empty_input();

    // Example 2: Handling truncated varint
    handle_truncated_varint();

    // Example 3: Handling invalid varint encoding
    handle_invalid_encoding();

    // Example 4: Using EncodedBytes for validation
    use_validated_bytes();

    // Example 5: Error recovery patterns
    error_recovery();

    // Example 6: Error source inspection
    inspect_error_source();
}

/// Example 1: Handling empty input
fn handle_empty_input() {
    println!("1. Handling Empty Input");
    println!("-----------------------");

    let empty: &[u8] = &[];

    match u32::try_decode_from(empty) {
        Ok((value, _)) => {
            println!("Unexpectedly decoded: {}", value);
        }
        Err(e) => {
            println!("✓ Correctly caught error: {}", e);
            println!("  Error type: UnsignedVarintDecode");
        }
    }

    println!();
}

/// Example 2: Handling truncated varint
fn handle_truncated_varint() {
    println!("2. Handling Truncated Varint");
    println!("-----------------------------");

    // 0x80 has continuation bit set, indicating more bytes follow
    // but there are no more bytes - this is truncated
    let truncated = vec![0x80];

    println!("Attempting to decode truncated varint: {:?}", truncated);

    match u16::try_decode_from(&truncated) {
        Ok((value, _)) => {
            println!("Unexpectedly decoded: {}", value);
        }
        Err(e) => {
            println!("✓ Correctly caught error: {}", e);
            println!("  This protects against malformed data");
        }
    }

    println!();
}

/// Example 3: Handling invalid varint encoding
fn handle_invalid_encoding() {
    println!("3. Handling Invalid Varint Encoding");
    println!("------------------------------------");

    // All bytes with continuation bit set (would overflow)
    let invalid = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];

    println!("Attempting to decode invalid varint with all continuation bits");

    match u64::try_decode_from(&invalid) {
        Ok((value, _)) => {
            println!("Unexpectedly decoded: {}", value);
        }
        Err(e) => {
            println!("✓ Correctly caught error: {}", e);
            println!("  This protects against overflow attacks");
        }
    }

    println!();
}

/// Example 4: Using EncodedBytes for validation
fn use_validated_bytes() {
    println!("4. Using EncodedBytes for Validation");
    println!("-------------------------------------");

    // Valid data
    let valid_data = vec![42u8];
    match EncodedBytes::try_from(valid_data.clone()) {
        Ok(encoded) => {
            println!("✓ Valid data accepted: {:?}", valid_data);
            println!("  EncodedBytes length: {}", encoded.len());
        }
        Err(e) => {
            println!("Unexpectedly rejected: {}", e);
        }
    }

    // Invalid data - empty
    let empty_data: Vec<u8> = vec![];
    match EncodedBytes::try_from(empty_data) {
        Ok(_) => {
            println!("Unexpectedly accepted empty data");
        }
        Err(e) => {
            println!("✓ Empty data rejected: {}", e);
        }
    }

    // Invalid data - truncated
    let truncated_data = vec![0x80];
    match EncodedBytes::try_from(truncated_data.clone()) {
        Ok(_) => {
            println!("Unexpectedly accepted truncated data");
        }
        Err(e) => {
            println!("✓ Truncated data rejected: {}", e);
            println!("  Attempted data: {:?}", truncated_data);
        }
    }

    println!();
}

/// Example 5: Error recovery patterns
fn error_recovery() {
    println!("5. Error Recovery Patterns");
    println!("--------------------------");

    // Try multiple decoding strategies
    let data = vec![0xFF, 0xFF, 0x03]; // Valid u16, might fail for u8

    println!("Attempting to decode {:?}...", data);

    // Try as u8 first
    match u8::try_decode_from(&data) {
        Ok((value, _)) => {
            println!("Decoded as u8: {}", value);
        }
        Err(_) => {
            println!("Failed to decode as u8, trying u16...");

            // Fall back to u16
            match u16::try_decode_from(&data) {
                Ok((value, _)) => {
                    println!("✓ Successfully decoded as u16: {}", value);
                }
                Err(e) => {
                    println!("Failed to decode as u16: {}", e);
                }
            }
        }
    }

    println!();
}

/// Example 6: Error source inspection
fn inspect_error_source() {
    println!("6. Error Source Inspection");
    println!("--------------------------");

    let invalid = vec![0x80, 0x80, 0x80]; // Truncated varint

    match u32::try_decode_from(&invalid) {
        Ok((value, _)) => {
            println!("Unexpectedly decoded: {}", value);
        }
        Err(e) => {
            println!("Error occurred: {}", e);

            // Pattern match on specific error types
            match &e {
                Error::UnsignedVarintDecode { source } => {
                    println!("  Error type: UnsignedVarintDecode");
                    println!("  Source error: {}", source);
                    println!("  This error comes from the unsigned-varint crate");
                }
                // Error is marked #[non_exhaustive] so we need a catch-all
                _ => {
                    println!("  Unknown error variant");
                }
            }

            // Check if error has a source (for error chaining)
            if let Some(source) = std::error::Error::source(&e) {
                println!("  Error source chain: {}", source);
            }
        }
    }

    println!();
}

/// Helper function showing how to wrap multitrait errors in application-specific errors
#[allow(dead_code)]
mod application_errors {
    use multitrait::Error as MultitraitError;

    #[derive(Debug)]
    pub enum AppError {
        InvalidData(MultitraitError),
        Other(String),
    }

    impl std::fmt::Display for AppError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                AppError::InvalidData(e) => write!(f, "Invalid data: {}", e),
                AppError::Other(s) => write!(f, "{}", s),
            }
        }
    }

    impl std::error::Error for AppError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                AppError::InvalidData(e) => Some(e),
                AppError::Other(_) => None,
            }
        }
    }

    impl From<MultitraitError> for AppError {
        fn from(e: MultitraitError) -> Self {
            AppError::InvalidData(e)
        }
    }
}
