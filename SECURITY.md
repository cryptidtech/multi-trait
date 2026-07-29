# Security Policy

## Overview

The `multi-trait` crate gives foundational traits for encoding and decoding
multiformats types. It uses unsigned varint encoding. This document describes
the security properties and the threat model of the crate.

## Security Properties

### Memory Safety

- No `unsafe` code. `#![deny(unsafe_code)]` is set at the crate root. The
  crate uses only safe Rust abstractions.
- No buffer overflows. All buffer operations use safe indexing and slicing.
- No use-after-free. Lifetimes keep references valid.
- No data races. All types are `Send + Sync` with no shared mutable state.

### Input Validation

- Malformed data rejection. Invalid varint encodings are detected and
  rejected with an error.
- Truncated data detection. Incomplete varints are rejected before
  processing.
- Boundary checking. All operations check buffer boundaries before access.
- Type-level validation. `EncodedBytes` gives data validity at the type
  level.

### Denial of Service Resistance

#### Protected Against

- Excessive memory allocation for integer encoding. Integer encoding uses
  at most 19 bytes (for `u128`).
- Stack overflow. No recursive operations. All encoding and decoding is
  iterative.
- Infinite loops. All loops have deterministic bounds set by the input
  size.
- Panic-based DoS. Tests check that no panic occurs on malicious input.

#### Bounded Operations

| Operation | Maximum Size | Notes |
|-----------|--------------|-------|
| `EncodeInto` for integers | 19 bytes | Maximum varint size for `u128` |
| `EncodeInto` for `[u8; N]` | N bytes | Raw byte copy. Not a varint. |
| `EncodeIntoArray` | 19 bytes | Stack-allocated. No heap. |
| `EncodeIntoBuffer` | 19 bytes per integer | Appends to an existing buffer. |
| `TryDecodeFrom` for integers | Input-dependent | Zero allocation. Bounded by input. |
| `TryDecodeFrom` for `[u8; N]` | N bytes | Reads exactly N bytes from the input. |
| `EncodedBytes` | Arbitrary | Validated at construction. |

### Integer Overflow Protection

- No arithmetic overflow. All operations delegate to the `unsigned-varint`
  crate. It handles overflow.
- Type-safe conversions. Decoding checks that values fit in the target
  type.
- Maximum value testing. Tests cover `TYPE::MAX` values for all types.

## Threat Model

### In Scope

The crate handles:

1. Untrusted input data from network sources, files, or user input.
2. Maliciously crafted varint sequences.
3. Extreme values at type boundaries (0 and MAX).
4. Truncated or incomplete data from interrupted transmissions.
5. Concurrent access from multiple threads.

### Out of Scope

The crate does not protect against:

1. Side-channel attacks. No constant-time guarantees. Timing may leak
   information.
2. Cryptographic security. This is an encoding library, not a crypto
   library.
3. Resource exhaustion. Callers must set rate limits and quotas.
4. Semantic validation. Callers must check that decoded values are
   semantically correct.
5. Physical attacks. No protection against hardware-level attacks.

## Security Guarantees

### What the Crate Guarantees

- No panics on invalid input. All error conditions return `Result`.
- No buffer overflows. All indexing is bounds-checked.
- No undefined behavior. `#![deny(unsafe_code)]` forbids unsafe code.
- Deterministic behavior. The same input always gives the same output.
- Error transparency. All errors give source chains for debugging.
- Type safety. Invalid states are not representable.

### What the Crate Does Not Guarantee

- Constant-time operations. Encoding and decoding time may vary with the
  input.
- Resource limits. Callers must set quotas.
- Side-channel resistance. The crate is not for cryptographic use.
- Backward compatibility with bugs. Security fixes may break buggy code.

## Dependencies

The crate depends on:

- `unsigned-varint` (v0.8). Gives the varint encoding and decoding.
- `thiserror` (v2.0). Error type derivation.

Dev dependencies:

- `proptest` (v1.4). Property-based testing.
- `criterion` (v0.5). Benchmarking.

### Dependency Security

- The `unsigned-varint` crate is used in the IPFS and libp2p ecosystem.
- This crate relies on its security properties for varint parsing.
- Any security issue in `unsigned-varint` affects this crate.

## Vulnerability Reporting

### Report a Vulnerability

If you find a security vulnerability in this crate, report it as follows:

1. Do not open a public GitHub issue.
2. Email the maintainers at the address in `Cargo.toml`.
3. Include a description of the vulnerability.
4. Include steps to reproduce it.
5. Include the potential impact.
6. Include a suggested fix if you have one.

### Response Timeline

- Initial response: within 48 hours.
- Vulnerability assessment: within 1 week.
- Fix development: depends on severity.
- Public disclosure: after the fix is released. Coordinated disclosure.

## Security Testing

### Test Coverage

The crate has these security tests:

- 13 malicious input tests for attack vectors.
- 6 property tests with random inputs.
- Edge case tests for boundary conditions, max values, and empty inputs.
- Regression tests for all discovered bugs.

### Run Security Tests

```bash
# Run all tests
cargo test

# Run only security tests
cargo test security_

# Run property tests with more cases
cargo test -- --ignored --test-threads=1
```

### Continuous Testing

All pull requests must pass:

- All unit tests.
- All integration tests.
- All property-based tests.
- Clippy with no warnings.
- Format check.

## Security Assumptions

### Assumptions About Callers

The crate assumes that callers will:

1. Validate decoded values. Check that decoded values make sense.
2. Set resource limits. Enforce quotas on input size and decode operations.
3. Handle errors. Do not ignore or unwrap `Result` in production code.
4. Use the right types. Pick the smallest type that can hold the data.

### Assumptions About the Environment

1. The Rust standard library is correct.
2. The dependencies are secure.
3. The compiler is correct.
4. The platform is not compromised.

## Best Practices

### For Users of This Crate

1. Always handle errors. Do not unwrap in production code.

```rust
// Bad
let (value, _) = u32::try_decode_from(untrusted).unwrap();

// Good
let (value, _) = u32::try_decode_from(untrusted)?;
```

2. Validate semantic correctness. Type checking is not enough.

```rust
let (port, _) = u16::try_decode_from(data)?;
if port == 0 {
    return Err("port cannot be zero");
}
```

3. Limit input size. Prevent resource exhaustion.

```rust
if input.len() > MAX_MESSAGE_SIZE {
    return Err("input too large");
}
```

4. Use validated types. Use `EncodedBytes` for pre-validated data.

```rust
fn process(data: EncodedBytes) {
    // The data is valid. No re-validation needed.
}
```

### For Contributors

1. Do not use `unsafe`. The crate enforces `#![deny(unsafe_code)]`.
2. Add tests for new features. Include security test cases.
3. Document security implications. Explain potential misuse.
4. Consider DoS vectors. Analyze every new feature for DoS potential.

## Version History

### Security-Relevant Changes

#### v1.0.5 (Current)
- Rewrote documentation in ASD-STE100 strict mode.
- Fixed inaccuracies in `README.md` and `SECURITY.md`.
- No known vulnerabilities.

#### v1.0.4
- Removed the unmaintained `serde_cbor` dev-dependency.
- Removed the redundant `unsafe impl Send/Sync` for `EncodedBytes`.
- No known vulnerabilities.

#### v1.0.3
- Added `#![deny(unsafe_code)]` and clippy lint configuration.
- Added MSRV declaration and CI jobs for audit, fmt, and clippy.
- No known vulnerabilities.

#### v1.0.0
- Initial production release.
- Validation in `EncodedBytes`.
- No known vulnerabilities.

## Audit Status

This crate has not had a professional security audit. Review the code
before you use it in security-critical applications.

## License

Security issues are handled under the same Apache-2.0 license as the crate.