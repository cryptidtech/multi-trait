# Security Policy

## Overview

The `multitrait` crate provides foundational traits for encoding and decoding multiformats types using unsigned varint encoding. This document outlines the security properties, threat model, and guarantees of this crate.

## Security Properties

### Memory Safety

- **No `unsafe` code**: The crate contains zero unsafe blocks, relying entirely on safe Rust abstractions
- **No buffer overflows**: All buffer operations use safe indexing and slicing
- **No use-after-free**: Lifetimes ensure references remain valid
- **No data races**: All types are `Send + Sync` with no mutable shared state

### Input Validation

- **Malformed data rejection**: Invalid varint encodings are detected and rejected with proper errors
- **Truncated data detection**: Incomplete varints are rejected before processing
- **Boundary checking**: All operations validate buffer boundaries before access
- **Type-level validation**: `EncodedBytes` newtype ensures data validity at the type level

### Denial of Service (DoS) Resistance

#### Protected Against

- **Excessive memory allocation**: Encoding uses bounded allocations (max 19 bytes for u128)
- **Stack overflow**: No recursive operations; all encoding/decoding is iterative
- **Infinite loops**: All loops have deterministic bounds based on input size
- **Panic-based DoS**: Comprehensive testing ensures no panics on malicious input

#### Bounded Operations

| Operation | Maximum Size | Notes |
|-----------|--------------|-------|
| `EncodeInto` | 19 bytes | Maximum varint size for u128 |
| `EncodeIntoArray` | 19 bytes | Stack-allocated, no heap |
| `TryDecodeFrom` | Input-dependent | Zero allocation, bounded by input |
| `EncodedBytes` | Arbitrary | Validated at construction |

### Integer Overflow Protection

- **No arithmetic overflow**: All operations delegate to `unsigned-varint` crate which handles overflow
- **Type-safe conversions**: Decoding validates that values fit in target type
- **Maximum value testing**: Comprehensive tests for `TYPE::MAX` values

## Threat Model

### In Scope

This crate is designed to handle:

1. **Untrusted input data** from network sources, files, or user input
2. **Maliciously crafted varint sequences** designed to trigger bugs
3. **Extreme values** at type boundaries (0, MAX)
4. **Truncated or incomplete data** from interrupted transmissions
5. **Concurrent access** from multiple threads

### Out of Scope

This crate does NOT protect against:

1. **Side-channel attacks**: No constant-time guarantees; timing may leak information
2. **Cryptographic security**: This is an encoding library, not a crypto library
3. **Resource exhaustion**: Callers must implement rate limiting and quotas
4. **Semantic validation**: Callers must validate that decoded values are semantically correct
5. **Physical attacks**: No protection against hardware-level attacks

## Security Guarantees

### What We Guarantee

✅ **No panics on invalid input**: All error conditions return `Result`
✅ **No buffer overflows**: All indexing is bounds-checked
✅ **No undefined behavior**: Zero unsafe code
✅ **Deterministic behavior**: Same input always produces same output
✅ **Error transparency**: All errors provide source chains for debugging
✅ **Type safety**: Invalid states are unrepresentable

### What We Do NOT Guarantee

❌ **Constant-time operations**: Encoding/decoding time may vary with input
❌ **Resource limits**: Callers must enforce quotas
❌ **Side-channel resistance**: Not designed for cryptographic use
❌ **Backward compatibility with bugs**: Security fixes may break buggy code

## Dependencies

This crate depends on:

- **`unsigned-varint` (v0.8)**: Provides the underlying varint encoding/decoding
- **`thiserror` (v2.0)**: Error type derivation
- **`proptest` (v1.4)**: Property-based testing (dev dependency)
- **`criterion` (v0.5)**: Benchmarking (dev dependency)

### Dependency Security

- The `unsigned-varint` crate is widely used in the IPFS/libp2p ecosystem
- We rely on its security properties for varint parsing
- Any security issues in `unsigned-varint` affect this crate

## Vulnerability Reporting

### Reporting a Vulnerability

If you discover a security vulnerability in this crate, please report it responsibly:

1. **Do NOT open a public GitHub issue** for security vulnerabilities
2. **Email the maintainers** at the address in `Cargo.toml`
3. **Include**:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### Response Timeline

- **Initial response**: Within 48 hours
- **Vulnerability assessment**: Within 1 week
- **Fix development**: Depends on severity
- **Public disclosure**: After fix is released (coordinated disclosure)

### Severity Classification

#### Critical (CVSS 9.0-10.0)
- Remote code execution
- Memory corruption
- Data exfiltration

#### High (CVSS 7.0-8.9)
- Denial of service (crash)
- Information disclosure
- Authentication bypass

#### Medium (CVSS 4.0-6.9)
- Denial of service (resource exhaustion)
- Logic errors affecting correctness

#### Low (CVSS 0.1-3.9)
- Minor issues with limited impact

## Security Testing

### Test Coverage

The crate includes comprehensive security tests:

- **Malicious input tests**: 13 targeted tests for attack vectors
- **Fuzzing-style property tests**: 6 property tests with random inputs
- **Edge case tests**: Boundary conditions, max values, empty inputs
- **Regression tests**: All discovered bugs have test coverage

### Running Security Tests

```bash
# Run all tests including security tests
cargo test

# Run only security tests
cargo test security_

# Run property-based tests with more cases
cargo test -- --ignored --test-threads=1
```

### Continuous Testing

All pull requests must pass:
- ✅ All unit tests
- ✅ All integration tests
- ✅ All property-based tests
- ✅ Clippy with no warnings
- ✅ Format check

## Security Assumptions

### Assumptions About Callers

This crate assumes callers will:

1. **Validate decoded values**: Check that decoded values make sense semantically
2. **Implement resource limits**: Enforce quotas on input size and decode operations
3. **Handle errors properly**: Don't ignore or unwrap Results in production code
4. **Use appropriate types**: Choose the smallest type that can represent your data

### Assumptions About Environment

1. **Rust standard library is correct**: We trust std/core/alloc
2. **Dependencies are secure**: We trust unsigned-varint and thiserror
3. **Compiler is correct**: We trust rustc and LLVM
4. **Platform is not compromised**: No hardware-level attacks

## Best Practices

### For Users of This Crate

1. **Always handle errors**: Never unwrap in production code
   ```rust
   // BAD
   let (value, _) = u32::try_decode_from(untrusted).unwrap();

   // GOOD
   let (value, _) = u32::try_decode_from(untrusted)?;
   ```

2. **Validate semantic correctness**: Type checking isn't enough
   ```rust
   let (port, _) = u16::try_decode_from(data)?;
   if port == 0 {
       return Err(Error::InvalidPort);
   }
   ```

3. **Limit input size**: Prevent resource exhaustion
   ```rust
   if input.len() > MAX_MESSAGE_SIZE {
       return Err(Error::TooLarge);
   }
   ```

4. **Use validated types**: Prefer `EncodedBytes` for pre-validated data
   ```rust
   fn process(data: EncodedBytes) {
       // Data is guaranteed valid, no re-validation needed
   }
   ```

### For Contributors

1. **Never use `unsafe`**: This crate has zero unsafe code by policy
2. **Add tests for new features**: Include security test cases
3. **Document security implications**: Explain potential misuse
4. **Consider DoS vectors**: Every new feature should be analyzed for DoS potential

## Version History

### Security-Relevant Changes

#### v1.0.1 (Current)
- Added `EncodeIntoBuffer` and `EncodeIntoArray` traits (Phase 5)
- Enhanced security test coverage (Phase 6)
- No known vulnerabilities

#### v1.0.0
- Initial production release
- Comprehensive validation in `EncodedBytes`
- Full test coverage
- No known vulnerabilities

## Audit Status

This crate has NOT undergone a professional security audit. Use in security-critical applications should be preceded by appropriate review.

## License

Security issues are handled under the same Apache-2.0 license as the crate itself.

## Acknowledgments

Security testing and review by the community is welcomed and appreciated. Thank you to all contributors who help keep this crate secure.

---

**Last Updated**: 2025-10-08
**Document Version**: 1.0
