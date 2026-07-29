[![](https://img.shields.io/badge/made%20by-Cryptid%20Technologies-gold.svg?style=flat-square)][CRYPTID]
[![](https://img.shields.io/badge/project-provenance-purple.svg?style=flat-square)][PROVENANCE]
[![](https://img.shields.io/badge/project-multiformats-blue.svg?style=flat-square)][MULTIFORMATS]
![](https://github.com/cryptidtech/multi-trait/actions/workflows/rust.yml/badge.svg)

# multi-trait

Common traits for multiformats types in Rust. The crate gives encoding,
decoding, and null value traits with zero-copy decoding and `no_std` support.

## Features

- Varint encoding with minimal allocations.
- Zero-copy decoding. `TryDecodeFrom` returns the remaining bytes.
- Zero-allocation encoding. `EncodeIntoBuffer` reuses an existing buffer.
- Stack-based encoding. `EncodeIntoArray` for `no_std` and embedded systems.
- `no_std` support with `alloc`.
- Validated newtype. `EncodedBytes` gives type-level guarantees.
- `#![deny(unsafe_code)]` set at the crate root.
- All types are `Send + Sync`.
- 155 tests: unit, property-based, security, concurrency, and round-trip.

## Install

Add this to your `Cargo.toml`:

```toml
[dependencies]
multi-trait = "1.1"
```

For `no_std` environments:

```toml
[dependencies]
multi-trait = { version = "1.1", default-features = false }
```

MSRV: Rust 1.85 (Edition 2024).

## Quick Start

```rust
use multi_trait::{EncodeInto, TryDecodeFrom};

// Encode a value to varint bytes
let value = 42u32;
let encoded = value.encode_into();
println!("Encoded {} as {:?}", value, encoded);

// Decode the bytes back to the value
let (decoded, remaining) = u32::try_decode_from(&encoded).unwrap();
assert_eq!(decoded, value);
assert!(remaining.is_empty());
```

## Core Traits

### Encoding Traits

#### `EncodeInto`

Encode a value into a varint `Vec<u8>`. Use this for one-off encoding.

```rust
use multi_trait::EncodeInto;

let value = 1000u16;
let bytes = value.encode_into(); // Allocates a new Vec<u8>
```

#### `EncodeIntoBuffer`

Encode values into an existing buffer with no allocation. Use this in hot
paths or when you encode multiple values.

```rust
use multi_trait::EncodeIntoBuffer;

let mut buffer = Vec::with_capacity(100);

42u8.encode_into_buffer(&mut buffer);
1000u16.encode_into_buffer(&mut buffer);
100_000u32.encode_into_buffer(&mut buffer);

println!("Encoded {} bytes total", buffer.len());
```

#### `EncodeIntoArray`

Encode a value into a stack-allocated array. Use this in `no_std` or
real-time systems.

```rust
use multi_trait::EncodeIntoArray;

let (array, len) = 42u8.encode_into_array();
assert_eq!(&array[..len], &[42]);

assert_eq!(<u32 as EncodeIntoArray>::MAX_ENCODED_SIZE, 5);
```

### Decoding Trait

#### `TryDecodeFrom`

Decode a value from a byte slice. Returns the value and the remaining bytes.
No allocation occurs.

```rust
use multi_trait::TryDecodeFrom;

let bytes = vec![0xFF, 0xFF, 0x03]; // Varint encoding of 65535
let (value, remaining) = u16::try_decode_from(&bytes).unwrap();
assert_eq!(value, 65535);
assert!(remaining.is_empty());

// Decode multiple values from one buffer
let bytes = vec![0x01, 0x02, 0x03];
let (first, rest) = u8::try_decode_from(&bytes).unwrap();
let (second, rest) = u8::try_decode_from(rest).unwrap();
let (third, rest) = u8::try_decode_from(rest).unwrap();
assert_eq!((first, second, third), (1, 2, 3));
```

### Null Value Traits

#### `Null`

Define and check for a null or sentinel value.

```rust
use multi_trait::Null;

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

Fallible version of `Null`. Use it for types that need validation.

```rust
use multi_trait::TryNull;

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

A validated newtype for varint-encoded byte sequences. Construction checks
that the bytes are a valid encoding.

```rust
use multi_trait::EncodedBytes;

let valid = vec![42u8];
let encoded = EncodedBytes::try_from(valid).unwrap();

let invalid = vec![0x80]; // Truncated varint
assert!(EncodedBytes::try_from(invalid).is_err());

fn process_encoded(data: EncodedBytes) {
    // The type guarantees the data is valid
    println!("Processing {} bytes", data.len());
}
```

## Error Handling

All decode operations return `Result` with a structured `Error` type:

```rust
use multi_trait::{TryDecodeFrom, Error};

let truncated = vec![0xFF]; // Incomplete varint
match u16::try_decode_from(&truncated) {
    Ok((value, _)) => println!("Decoded: {}", value),
    Err(Error::UnsignedVarintDecode { .. }) => {
        eprintln!("Decode failed");
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

### Error Types

The `Error` enum is `#[non_exhaustive]`. It has these variants:

- `UnsignedVarintDecode`: Varint decoding failed. The cause can be
  truncated data or an invalid encoding.
- `InsufficientData`: The input slice does not have enough bytes to decode
  the requested type.
- `InvalidEncoding`: The data is structurally invalid. This variant is
  for future use and custom validation.

All errors give source chains for debugging. Backtraces are available when
the `std` feature is on.

## Performance Guide

### Encoding Performance

Pick the encoding strategy for your use case:

1. `EncodeInto` — One allocation per call. Use this for one-off encodings.
2. `EncodeIntoBuffer` — Zero allocations when the buffer has capacity. Use
   this in hot paths or when you encode multiple values.
3. `EncodeIntoArray` — Zero heap allocations. Use this in `no_std` or
   real-time systems.

### Decoding Performance

- Zero allocations. Returns slice references to the input data.
- No data copy during decode.
- Efficient varint format checking.

### Encoded Sizes

Varint encoding uses 1 to 10 bytes for integers. The size depends on the
value.

- Values 0 to 127: 1 byte.
- Values 128 to 16,383: 2 bytes.
- Values 16,384 to 2,097,151: 3 bytes.

Maximum encoded sizes by type:

- `u8`, `bool`: 2 bytes.
- `u16`: 3 bytes.
- `u32`: 5 bytes.
- `u64`, `usize` (64-bit): 10 bytes.
- `u128`: 19 bytes.

## Thread Safety

All traits and types in this crate are `Send + Sync`. You can use them in
concurrent contexts.

All operations are lock-free. No mutable state is shared.

## `no_std` Support

The crate works in `no_std` environments with `alloc`:

```toml
[dependencies]
multi-trait = { version = "1.1", default-features = false }
```

Use `EncodeIntoArray` for heap-free encoding in embedded systems:

```rust
#![no_std]

extern crate alloc;
use alloc::vec::Vec;
use multi_trait::EncodeIntoArray;

let (array, len) = 42u8.encode_into_array();

let vec = Vec::from(&array[..len]);
```

## Feature Flags

- `std` (default). Enables standard library support. It enables
  `std::error::Error` implementation and backtrace support in errors.
  Disable it for `no_std` with `default-features = false`. The crate needs
  `alloc` when `std` is off.

## Supported Types

All traits are implemented for:

- `bool`: Encoded as 0 (false) or 1 (true).
- `u8`, `u16`, `u32`, `u64`, `u128`: Variable-length varint encoding.
- `usize`: Platform-dependent (32-bit or 64-bit).
- `[u8; N]`: Fixed-length byte arrays. `EncodeInto` encodes the raw bytes
  without a varint prefix. `TryDecodeFrom` reads exactly N bytes. Use this
  for BLS share identifiers and other fixed-size binary data.

`EncodeIntoBuffer` and `EncodeIntoArray` are implemented for `bool`,
`u8`, `u16`, `u32`, `u64`, `u128`, and `usize`. They do not support
`[u8; N]`.

## Examples

See the `examples/` directory for complete examples:

- `basic.rs` — Basic encoding and decoding.
- `error_handling.rs` — Error handling patterns.
- `custom_type.rs` — Implement the traits for custom types.
- `no_std.rs` — Use the crate in `no_std` environments.

Run an example:

```bash
cargo run --example basic
```

## Testing

The crate has 155 tests: unit, property-based, concurrency, security, and
edge case tests.

Run all tests:

```bash
cargo test
```

## Documentation

Generate and view the API documentation:

```bash
cargo doc --open
```

## License

Licensed under Apache-2.0. See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome. Before you submit a change, make sure:

- All tests pass (`cargo test`).
- The code is formatted (`cargo fmt`).
- No clippy warnings (`cargo clippy`).
- New features include tests and documentation.

## Links

- [Documentation](https://docs.rs/multi-trait)
- [Crates.io](https://crates.io/crates/multi-trait)
- [Repository](https://github.com/cryptidtech/multi-trait)
- [Multiformats](https://github.com/multiformats/multiformats/)

[CRYPTID]: https://cryptid.tech/
[PROVENANCE]: https://github.com/cryptidtech/provenance-specifications/
[MULTIFORMATS]: https://github.com/multiformats/multiformats/