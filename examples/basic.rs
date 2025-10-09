// SPDX-License-Identifier: Apache-2.0
//! Basic encoding and decoding example
//!
//! This example demonstrates the core functionality of the multitrait crate:
//! - Encoding values to bytes
//! - Decoding bytes back to values
//! - Sequential encoding and decoding
//! - Using different encoding strategies

use multitrait::{EncodeInto, EncodeIntoArray, EncodeIntoBuffer, TryDecodeFrom};

fn main() {
    println!("=== Multitrait Basic Example ===\n");

    // Example 1: Basic encoding and decoding
    basic_encode_decode();

    // Example 2: Sequential encoding and decoding
    sequential_operations();

    // Example 3: Buffer-based encoding (zero allocation)
    buffer_based_encoding();

    // Example 4: Stack-based encoding (no heap)
    stack_based_encoding();

    // Example 5: Understanding varint encoding efficiency
    varint_efficiency();
}

/// Example 1: Basic encoding and decoding
fn basic_encode_decode() {
    println!("1. Basic Encoding and Decoding");
    println!("--------------------------------");

    // Encode a simple value
    let value = 42u8;
    let encoded = value.encode_into();
    println!("Original value: {}", value);
    println!("Encoded bytes: {:?}", encoded);
    println!("Encoded length: {} byte(s)", encoded.len());

    // Decode it back
    let (decoded, remaining) = u8::try_decode_from(&encoded).unwrap();
    println!("Decoded value: {}", decoded);
    println!("Remaining bytes: {:?}", remaining);
    assert_eq!(value, decoded);
    assert!(remaining.is_empty());

    println!();
}

/// Example 2: Sequential encoding and decoding
fn sequential_operations() {
    println!("2. Sequential Encoding and Decoding");
    println!("------------------------------------");

    // Encode multiple values into one buffer
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&42u8.encode_into());
    buffer.extend_from_slice(&1000u16.encode_into());
    buffer.extend_from_slice(&100000u32.encode_into());

    println!("Encoded 3 values:");
    println!("  - 42 (u8)");
    println!("  - 1000 (u16)");
    println!("  - 100000 (u32)");
    println!("Total buffer size: {} bytes", buffer.len());
    println!("Buffer contents: {:?}", buffer);

    // Decode sequentially
    let (val1, rest) = u8::try_decode_from(&buffer).unwrap();
    let (val2, rest) = u16::try_decode_from(rest).unwrap();
    let (val3, rest) = u32::try_decode_from(rest).unwrap();

    println!("\nDecoded values:");
    println!("  - {} (u8)", val1);
    println!("  - {} (u16)", val2);
    println!("  - {} (u32)", val3);

    assert_eq!(val1, 42);
    assert_eq!(val2, 1000);
    assert_eq!(val3, 100000);
    assert!(rest.is_empty());

    println!();
}

/// Example 3: Buffer-based encoding (zero allocation)
fn buffer_based_encoding() {
    println!("3. Buffer-Based Encoding (Zero Allocation)");
    println!("-------------------------------------------");

    // Pre-allocate buffer
    let mut buffer = Vec::with_capacity(100);
    println!("Initial buffer capacity: {}", buffer.capacity());

    // Encode multiple values without allocating
    for i in 0u16..50 {
        i.encode_into_buffer(&mut buffer);
    }

    println!("Encoded 50 values (0-49)");
    println!("Buffer length: {} bytes", buffer.len());
    println!(
        "Buffer capacity: {} (no reallocation needed!)",
        buffer.capacity()
    );

    // Decode and verify
    let mut slice = &buffer[..];
    let mut count = 0;
    while !slice.is_empty() {
        let (value, remaining) = u16::try_decode_from(slice).unwrap();
        assert_eq!(value, count);
        slice = remaining;
        count += 1;
    }
    println!("Successfully decoded and verified {} values", count);

    println!();
}

/// Example 4: Stack-based encoding (no heap)
fn stack_based_encoding() {
    println!("4. Stack-Based Encoding (No Heap Allocation)");
    println!("---------------------------------------------");

    // Encode to stack-allocated array
    let value = 42u8;
    let (array, len) = value.encode_into_array();

    println!("Encoded {} to stack array", value);
    println!("Array contents (first {} bytes): {:?}", len, &array[..len]);
    println!(
        "Maximum array size for u8: {} bytes",
        <u8 as EncodeIntoArray>::MAX_ENCODED_SIZE
    );

    // Show maximum sizes for different types
    println!("\nMaximum encoded sizes:");
    println!(
        "  - bool:  {} byte(s)",
        <bool as EncodeIntoArray>::MAX_ENCODED_SIZE
    );
    println!(
        "  - u8:    {} byte(s)",
        <u8 as EncodeIntoArray>::MAX_ENCODED_SIZE
    );
    println!(
        "  - u16:   {} byte(s)",
        <u16 as EncodeIntoArray>::MAX_ENCODED_SIZE
    );
    println!(
        "  - u32:   {} byte(s)",
        <u32 as EncodeIntoArray>::MAX_ENCODED_SIZE
    );
    println!(
        "  - u64:   {} byte(s)",
        <u64 as EncodeIntoArray>::MAX_ENCODED_SIZE
    );
    println!(
        "  - u128:  {} byte(s)",
        <u128 as EncodeIntoArray>::MAX_ENCODED_SIZE
    );
    println!(
        "  - usize: {} byte(s)",
        <usize as EncodeIntoArray>::MAX_ENCODED_SIZE
    );

    println!();
}

/// Example 5: Understanding varint encoding efficiency
fn varint_efficiency() {
    println!("5. Varint Encoding Efficiency");
    println!("------------------------------");

    // Show how different values encode to different lengths
    let test_values = vec![0u32, 127, 128, 255, 16383, 16384, u32::MAX];

    println!("Value encoding efficiency:");
    println!("{:>12} | {:>6} | Encoded (hex)", "Value", "Bytes");
    println!("{0:-<12}-+-{0:-<6}-+-{0:-<30}", "");

    for value in test_values {
        let encoded = value.encode_into();
        let hex_str: String = encoded
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        println!("{:>12} | {:>6} | {}", value, encoded.len(), hex_str);
    }

    println!("\nKey insight: Smaller values take fewer bytes!");
    println!("  - Values 0-127: 1 byte");
    println!("  - Values 128-16,383: 2 bytes");
    println!("  - Values 16,384-2,097,151: 3 bytes");
    println!("  - etc.");

    println!();
}
