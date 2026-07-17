// SPDX-License-Identifier: Apache-2.0
//! `no_std` usage example
//!
//! This example demonstrates how to use the multi-trait crate in a `no_std`
//! environment. While this example file runs with std (since examples require it),
#![allow(clippy::items_after_statements, clippy::unreadable_literal)]
//! it shows the patterns you would use in actual `no_std` code.
//!
//! To use multi-trait in a `no_std` environment:
//!
//! 1. In your Cargo.toml:
//!    ```toml
//!    [dependencies]
//!    multi-trait = { version = "1.0", default-features = false }
//!    ```
//!
//! 2. In your code:
//!    ```rust
//!    #![no_std]
//!    extern crate alloc;
//!    use alloc::vec::Vec;
//!    ```

// Note: In actual no_std code, you would use:
// #![no_std]
// extern crate alloc;

use multi_trait::{EncodeInto, EncodeIntoArray, EncodeIntoBuffer, TryDecodeFrom};

fn main() {
    println!("=== Multitrait no_std Usage Patterns ===\n");
    println!("Note: This example demonstrates no_std patterns");
    println!("but runs with std for example execution.\n");

    // Example 1: Stack-based encoding (zero heap)
    stack_based_encoding();

    // Example 2: Minimal heap usage
    minimal_heap_usage();

    // Example 3: Const generics for fixed-size buffers
    fixed_size_buffers();

    // Example 4: Decoding without allocation
    zero_allocation_decoding();
}

/// Example 1: Stack-based encoding (zero heap allocation)
///
/// This is ideal for embedded systems and `no_std` environments where
/// heap allocation should be avoided or is not available.
fn stack_based_encoding() {
    println!("1. Stack-Based Encoding (Zero Heap)");
    println!("------------------------------------");

    // Encode directly to stack-allocated array
    let value = 42u8;
    let (array, len) = value.encode_into_array();

    println!("Encoded {value} to stack array");
    println!("Used {} byte(s) out of maximum {}", len, array.len());
    println!("Array contents: {:?}", &array[..len]);

    // Decode from stack array
    let (decoded, _) = u8::try_decode_from(&array[..len]).unwrap();
    println!("Decoded: {decoded}");
    assert_eq!(value, decoded);

    // Show different types
    println!("\nStack encoding for different types:");

    let (_array, len) = 1000u16.encode_into_array();
    println!("  u16 (1000): {len} byte(s)");

    let (_array, len) = 100000u32.encode_into_array();
    println!("  u32 (100000): {len} byte(s)");

    let (_array, len) = u64::MAX.encode_into_array();
    println!("  u64::MAX: {len} byte(s) (maximum for u64)");

    println!();
}

/// Example 2: Minimal heap usage
///
/// When you need Vec but want to minimize allocations,
/// use `EncodeIntoBuffer` with pre-allocated capacity.
fn minimal_heap_usage() {
    println!("2. Minimal Heap Usage");
    println!("---------------------");

    // In no_std, you would use: use alloc::vec::Vec;
    // Pre-allocate buffer to avoid multiple allocations
    let mut buffer = Vec::with_capacity(100);
    let initial_capacity = buffer.capacity();

    println!("Pre-allocated buffer capacity: {initial_capacity}");

    // Encode multiple values without reallocating
    for i in 0u16..30 {
        i.encode_into_buffer(&mut buffer);
    }

    println!("Encoded 30 values");
    println!("Buffer length: {} bytes", buffer.len());
    println!("Buffer capacity: {} (no reallocation!)", buffer.capacity());
    assert_eq!(buffer.capacity(), initial_capacity);

    // Decode without allocation
    let mut slice = &buffer[..];
    let mut count = 0;
    while !slice.is_empty() {
        let (value, remaining) = u16::try_decode_from(slice).unwrap();
        assert_eq!(value, count);
        slice = remaining;
        count += 1;
    }
    println!("Decoded {count} values without allocation");

    println!();
}

/// Example 3: Fixed-size buffers for embedded systems
///
/// Using fixed-size arrays on the stack is common in embedded systems
/// to have deterministic memory usage.
fn fixed_size_buffers() {
    println!("3. Fixed-Size Buffers for Embedded");
    println!("-----------------------------------");

    // Create a fixed-size buffer on the stack
    const BUFFER_SIZE: usize = 64;
    let mut buffer = [0u8; BUFFER_SIZE];
    let mut offset = 0;

    println!("Fixed-size buffer: {BUFFER_SIZE} bytes");

    // Encode values into the fixed buffer
    let values = [10u16, 20, 30, 40, 50];
    for value in &values {
        let (array, len) = value.encode_into_array();
        buffer[offset..offset + len].copy_from_slice(&array[..len]);
        offset += len;
    }

    println!("Encoded {} values", values.len());
    println!("Used {offset} bytes out of {BUFFER_SIZE} available");

    // Decode from the fixed buffer
    let mut slice = &buffer[..offset];
    let mut decoded_values = Vec::new(); // In no_std: use alloc::vec::Vec
    while !slice.is_empty() {
        let (value, remaining) = u16::try_decode_from(slice).unwrap();
        decoded_values.push(value);
        slice = remaining;
    }

    println!("Decoded values: {decoded_values:?}");
    assert_eq!(decoded_values.as_slice(), &values);

    println!();
}

/// Example 4: Zero-allocation decoding
///
/// Decoding never allocates - it only returns references to the input buffer.
/// This is perfect for `no_std` and resource-constrained environments.
fn zero_allocation_decoding() {
    println!("4. Zero-Allocation Decoding");
    println!("----------------------------");

    // Create some encoded data
    let mut data = Vec::new(); // In no_std: use alloc::vec::Vec
    for i in 0u32..10 {
        data.extend_from_slice(&i.encode_into());
    }

    println!("Encoded 10 values in {} bytes", data.len());
    println!("Data: {data:?}");

    // Decode using only stack references - no allocation
    let mut slice = &data[..];
    let mut sum = 0u64;
    let mut count = 0;

    println!("\nDecoding values (no allocation):");
    while !slice.is_empty() {
        // try_decode_from returns references, never allocates
        let (value, remaining) = u32::try_decode_from(slice).unwrap();
        println!("  Value {count}: {value}");
        sum += u64::from(value);
        slice = remaining;
        count += 1;
    }

    println!("\nDecoded {count} values");
    println!("Sum: {sum}");

    // Verify
    let expected_sum: u64 = (0..10).sum();
    assert_eq!(sum, expected_sum);

    println!();
}

/// Example showing how to implement custom types for `no_std`
#[allow(dead_code)]
mod no_std_custom_types {
    // In actual no_std code:
    // use alloc::vec::Vec;
    use multi_trait::{EncodeIntoArray, EncodeIntoBuffer, TryDecodeFrom};

    /// A custom type that uses only stack encoding
    #[derive(Debug, PartialEq, Eq)]
    pub struct SensorReading {
        pub sensor_id: u8,
        pub value: u16,
    }

    impl SensorReading {
        /// Encode to stack array (no heap required)
        pub fn encode_to_array(&self) -> ([u8; 19], usize) {
            let (id_array, id_len) = self.sensor_id.encode_into_array();
            let (val_array, val_len) = self.value.encode_into_array();

            let mut result = [0u8; 19];
            result[..id_len].copy_from_slice(&id_array[..id_len]);
            result[id_len..id_len + val_len].copy_from_slice(&val_array[..val_len]);

            (result, id_len + val_len)
        }

        /// Decode from bytes (zero allocation)
        pub fn decode_from(bytes: &[u8]) -> Result<(Self, &[u8]), multi_trait::Error> {
            let (sensor_id, remaining) = u8::try_decode_from(bytes)?;
            let (value, remaining) = u16::try_decode_from(remaining)?;
            Ok((Self { sensor_id, value }, remaining))
        }

        /// Encode into existing buffer (minimal allocation)
        pub fn encode_into_buffer(&self, buffer: &mut Vec<u8>) {
            self.sensor_id.encode_into_buffer(buffer);
            self.value.encode_into_buffer(buffer);
        }
    }
}
