[![](https://img.shields.io/badge/made%20by-Cryptid%20Technologies-gold.svg?style=flat-square)][CRYPTID]
[![](https://img.shields.io/badge/project-provenance-purple.svg?style=flat-square)][PROVENANCE]
[![](https://img.shields.io/badge/project-multiformats-blue.svg?style=flat-square)][MULTIFORMATS]
![](https://github.com/cryptidtech/multitrait/actions/workflows/rust.yml/badge.svg)

# Multitrait

A lightweight, high-performance Rust library providing common traits for implementing [multiformats](https://github.com/multiformats/multiformats) types with zero-copy decoding and flexible encoding strategies.

## Features

- **🚀 High Performance**: Optimized varint encoding with minimal allocations
- **📦 Zero-Copy Decoding**: Parse data without unnecessary copying
- **🎯 Type Safety**: Validated newtypes for compile-time guarantees
- **🔧 Flexible Encoding**: Three encoding strategies for different use cases
- **🌐 no_std Support**: Works in embedded and constrained environments
- **🧵 Thread-Safe**: All traits are `Send + Sync` safe
- **📝 Well-Documented**: Comprehensive documentation with examples
- **✅ Thoroughly Tested**: 150+ tests including property-based and concurrency tests

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
multitrait = "1.0"
```

For `no_std` environments:

```toml
[dependencies]
multitrait = { version = "1.0", default-features = false }
```

## Quick Start

```rust
use multitrait::{EncodeInto, TryDecodeFrom};

// Encoding: Convert a value to compact varint bytes
let value = 42u32;
let encoded = value.encode_into();
println!("Encoded {} as {:?}", value, encoded);

// Decoding: Parse bytes back to original value
let (decoded, remaining) = u32::try_decode_from(&encoded).unwrap();
assert_eq!(decoded, value);
assert!(remaining.is_empty());
```

## Core Traits

### Encoding Traits

#### `EncodeInto`

Encode values into a compact varint `Vec<u8>`. Best for one-off encoding operations.

```rust
use multitrait::EncodeInto;

let value = 1000u16;
let bytes = value.encode_into(); // Allocates new Vec<u8>
```

#### `EncodeIntoBuffer`

Zero-allocation encoding into an existing buffer. Best for encoding multiple values or hot paths.

```rust
use multitrait::EncodeIntoBuffer;

let mut buffer = Vec::with_capacity(100);

// Encode multiple values with minimal allocations
42u8.encode_into_buffer(&mut buffer);
1000u16.encode_into_buffer(&mut buffer);
100000u32.encode_into_buffer(&mut buffer);

println!("Encoded {} bytes total", buffer.len());
```

#### `EncodeIntoArray`

Stack-based encoding for `no_std` environments. Returns a fixed-size array with the actual length.

```rust
use multitrait::EncodeIntoArray;

let (array, len) = 42u8.encode_into_array();
assert_eq!(&array[..len], &[42]);

// Maximum sizes known at compile time
assert_eq!(<u32 as EncodeIntoArray>::MAX_ENCODED_SIZE, 5);
```

### Decoding Trait

#### `TryDecodeFrom`

Fallibly decode values from byte slices with zero-copy semantics. Returns the decoded value and remaining unconsumed bytes.

```rust
use multitrait::TryDecodeFrom;

let bytes = vec![0xFF, 0xFF, 0x03]; // Varint encoding of 65535
let (value, remaining) = u16::try_decode_from(&bytes).unwrap();
assert_eq!(value, 65535);
assert!(remaining.is_empty());

// Sequential decoding from one buffer
let bytes = vec![0x01, 0x02, 0x03];
let (first, rest) = u8::try_decode_from(&bytes).unwrap();
let (second, rest) = u8::try_decode_from(rest).unwrap();
let (third, rest) = u8::try_decode_from(rest).unwrap();
assert_eq!((first, second, third), (1, 2, 3));
```

### Null Value Traits

#### `Null`

Define and check for null/sentinel values.

```rust
use multitrait::Null;

struct MyId(u64);

impl Null for MyId {
    fn null() -> Self {
        MyId(0)
    }

    fn is_null(&self) -> bool {
        self.0 == 0
    }
}

let null_id = MyId::null();
assert!(null_id.is_null());

let valid_id = MyId(12345);
assert!(!valid_id.is_null());
```

#### `TryNull`

Fallible version of `Null` for types requiring validation.

```rust
use multitrait::TryNull;

struct ValidatedId(u64);

impl TryNull for ValidatedId {
    type Error = &'static str;

    fn try_null() -> Result<Self, Self::Error> {
        Ok(ValidatedId(0))
    }

    fn is_null(&self) -> bool {
        self.0 == 0
    }
}
```

### Validated Types

#### `EncodedBytes`

A validated newtype for varint-encoded byte sequences. Provides compile-time guarantees that bytes represent valid encodings.

```rust
use multitrait::EncodedBytes;

// Validation happens at construction
let valid = vec![42u8];
let encoded = EncodedBytes::try_from(valid).unwrap();

// Invalid data is rejected
let invalid = vec![0x80]; // Truncated varint
assert!(EncodedBytes::try_from(invalid).is_err());

// Type system ensures valid data
fn process_encoded(data: EncodedBytes) {
    // No need to validate - type guarantees validity
    println!("Processing {} bytes", data.len());
}
```

## Error Handling

All decode operations return a `Result` with a structured `Error` type:

```rust
use multitrait::{TryDecodeFrom, Error};

let truncated = vec![0xFF]; // Incomplete varint
match u16::try_decode_from(&truncated) {
    Ok((value, _)) => println!("Decoded: {}", value),
    Err(Error::UnsignedVarintDecode { source }) => {
        eprintln!("Decode failed: {}", source);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

### Error Types

- `Error::UnsignedVarintDecode`: Varint decoding failed (truncated data, invalid encoding, etc.)

All errors include source chains for debugging and support backtraces when the `std` feature is enabled.

## Performance Guide

### Encoding Performance

Choose the right encoding strategy for your use case:

1. **`EncodeInto`** - Good for one-off encodings
   - Single allocation per call
   - Simple API
   - Use when encoding individual values

2. **`EncodeIntoBuffer`** - Best for multiple values
   - Zero allocations when buffer has capacity
   - Reusable buffer
   - Use in hot paths or when encoding multiple values

3. **`EncodeIntoArray`** - Best for embedded systems
   - Zero heap allocations (stack only)
   - Deterministic performance
   - Use in `no_std` or real-time systems

### Decoding Performance

- **Zero allocations**: Returns slice references to existing data
- **Zero copying**: No data duplication during decode
- **Constant-time validation**: Efficient varint format checking

### Benchmark Results

Varint encoding is highly efficient:
- Values 0-127: 1 byte
- Values 128-16,383: 2 bytes
- Values 16,384-2,097,151: 3 bytes
- And so on...

Maximum encoded sizes:
- `u8`, `bool`: 2 bytes max
- `u16`: 3 bytes max
- `u32`: 5 bytes max
- `u64`, `usize` (64-bit): 10 bytes max
- `u128`: 19 bytes max

## Thread Safety

All traits and types in this crate are `Send + Sync`, making them safe to use in concurrent contexts.

### Concurrency Patterns

All these patterns work safely:

```rust
use std::sync::Arc;
use std::thread;
use multitrait::{EncodeInto, TryDecodeFrom};

// Parallel encoding
let handles: Vec<_> = (0..10)
    .map(|i| {
        thread::spawn(move || {
            let value = i as u32 * 100;
            value.encode_into()
        })
    })
    .collect();

// Shared read access
let data = Arc::new(vec![42u8, 100, 200]);
let handles: Vec<_> = (0..4)
    .map(|_| {
        let data = Arc::clone(&data);
        thread::spawn(move || {
            let (value, _) = u8::try_decode_from(&data).unwrap();
            value
        })
    })
    .collect();
```

All operations are lock-free with no shared mutable state.

## no_std Support

This crate works in `no_std` environments with `alloc`:

```toml
[dependencies]
multitrait = { version = "1.0", default-features = false }
```

Use `EncodeIntoArray` for heap-free encoding in embedded systems:

```rust
#![no_std]

extern crate alloc;
use alloc::vec::Vec;
use multitrait::EncodeIntoArray;

// Stack-only encoding (no heap required for encoding)
let (array, len) = 42u8.encode_into_array();

// Only allocate if you need to store it
let vec = Vec::from(&array[..len]);
```

## Feature Flags

- **`std`** (default): Enables standard library support
  - Enables `std::error::Error` implementation
  - Enables backtrace support in errors
  - Disable for `no_std`: `default-features = false`

## Supported Types

All traits are implemented for:
- `bool`: Encoded as 0 (false) or 1 (true)
- `u8`, `u16`, `u32`, `u64`, `u128`: Variable-length encoding
- `usize`: Platform-dependent (32-bit or 64-bit)

## Examples

See the [`examples/`](examples/) directory for complete examples:

- [`basic.rs`](examples/basic.rs) - Basic encoding and decoding
- [`error_handling.rs`](examples/error_handling.rs) - Error handling patterns
- [`custom_type.rs`](examples/custom_type.rs) - Implementing traits for custom types
- [`no_std.rs`](examples/no_std.rs) - Using the crate in no_std environments

Run examples with:
```bash
cargo run --example basic
cargo run --example error_handling
cargo run --example custom_type
```

## Testing

The crate includes 150+ tests:
- Unit tests for all trait implementations
- Property-based tests with proptest
- Concurrency tests for thread safety
- Security tests for malicious inputs
- Edge case tests

Run tests with:
```bash
cargo test
```

## Documentation

Generate and view the full API documentation:
```bash
cargo doc --open
```

## License

Licensed under Apache-2.0. See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please ensure:
- All tests pass (`cargo test`)
- Code is formatted (`cargo fmt`)
- No clippy warnings (`cargo clippy`)
- New features include tests and documentation

## Links

- [Documentation](https://docs.rs/multitrait)
- [Crates.io](https://crates.io/crates/multitrait)
- [Repository](https://github.com/cryptidtech/multitrait)
- [Multiformats](https://github.com/multiformats/multiformats/)

[CRYPTID]: https://cryptid.tech/
[PROVENANCE]: https://github.com/cryptidtech/provenance-specifications/
[MULTIFORMATS]: https://github.com/multiformats/multiformats/
