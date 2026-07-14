// SPDX-License-Identifier: Apache-2.0
//! Benchmarks for encoding and decoding operations
//!
//! Run with: `cargo bench`

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use multi_trait::{EncodeInto, EncodeIntoArray, EncodeIntoBuffer, TryDecodeFrom};

/// Benchmark encoding operations for various integer types
fn bench_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");

    // u8 encoding
    group.bench_function("u8_small", |b| b.iter(|| black_box(42u8).encode_into()));

    group.bench_function("u8_max", |b| b.iter(|| black_box(u8::MAX).encode_into()));

    // u16 encoding
    group.bench_function("u16_small", |b| b.iter(|| black_box(42u16).encode_into()));

    group.bench_function("u16_max", |b| b.iter(|| black_box(u16::MAX).encode_into()));

    // u32 encoding
    group.bench_function("u32_small", |b| b.iter(|| black_box(42u32).encode_into()));

    group.bench_function("u32_max", |b| b.iter(|| black_box(u32::MAX).encode_into()));

    // u64 encoding
    group.bench_function("u64_small", |b| b.iter(|| black_box(42u64).encode_into()));

    group.bench_function("u64_max", |b| b.iter(|| black_box(u64::MAX).encode_into()));

    // u128 encoding
    group.bench_function("u128_small", |b| b.iter(|| black_box(42u128).encode_into()));

    group.bench_function("u128_max", |b| {
        b.iter(|| black_box(u128::MAX).encode_into())
    });

    // bool encoding
    group.bench_function("bool_true", |b| b.iter(|| black_box(true).encode_into()));

    group.bench_function("bool_false", |b| b.iter(|| black_box(false).encode_into()));

    group.finish();
}

/// Benchmark decoding operations for various integer types
fn bench_decoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");

    // Pre-encode values for decoding benchmarks
    let u8_small = 42u8.encode_into();
    let u8_max = u8::MAX.encode_into();
    let u16_small = 42u16.encode_into();
    let u16_max = u16::MAX.encode_into();
    let u32_small = 42u32.encode_into();
    let u32_max = u32::MAX.encode_into();
    let u64_small = 42u64.encode_into();
    let u64_max = u64::MAX.encode_into();
    let u128_small = 42u128.encode_into();
    let u128_max = u128::MAX.encode_into();
    let bool_true = true.encode_into();
    let bool_false = false.encode_into();

    // u8 decoding
    group.bench_function("u8_small", |b| {
        b.iter(|| u8::try_decode_from(black_box(&u8_small)).unwrap())
    });

    group.bench_function("u8_max", |b| {
        b.iter(|| u8::try_decode_from(black_box(&u8_max)).unwrap())
    });

    // u16 decoding
    group.bench_function("u16_small", |b| {
        b.iter(|| u16::try_decode_from(black_box(&u16_small)).unwrap())
    });

    group.bench_function("u16_max", |b| {
        b.iter(|| u16::try_decode_from(black_box(&u16_max)).unwrap())
    });

    // u32 decoding
    group.bench_function("u32_small", |b| {
        b.iter(|| u32::try_decode_from(black_box(&u32_small)).unwrap())
    });

    group.bench_function("u32_max", |b| {
        b.iter(|| u32::try_decode_from(black_box(&u32_max)).unwrap())
    });

    // u64 decoding
    group.bench_function("u64_small", |b| {
        b.iter(|| u64::try_decode_from(black_box(&u64_small)).unwrap())
    });

    group.bench_function("u64_max", |b| {
        b.iter(|| u64::try_decode_from(black_box(&u64_max)).unwrap())
    });

    // u128 decoding
    group.bench_function("u128_small", |b| {
        b.iter(|| u128::try_decode_from(black_box(&u128_small)).unwrap())
    });

    group.bench_function("u128_max", |b| {
        b.iter(|| u128::try_decode_from(black_box(&u128_max)).unwrap())
    });

    // bool decoding
    group.bench_function("bool_true", |b| {
        b.iter(|| bool::try_decode_from(black_box(&bool_true)).unwrap())
    });

    group.bench_function("bool_false", |b| {
        b.iter(|| bool::try_decode_from(black_box(&bool_false)).unwrap())
    });

    group.finish();
}

/// Benchmark round-trip operations (encode + decode)
fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");

    group.bench_function("u8", |b| {
        b.iter(|| {
            let value = black_box(42u8);
            let encoded = value.encode_into();
            let (decoded, _) = u8::try_decode_from(&encoded).unwrap();
            black_box(decoded)
        })
    });

    group.bench_function("u16", |b| {
        b.iter(|| {
            let value = black_box(42u16);
            let encoded = value.encode_into();
            let (decoded, _) = u16::try_decode_from(&encoded).unwrap();
            black_box(decoded)
        })
    });

    group.bench_function("u32", |b| {
        b.iter(|| {
            let value = black_box(42u32);
            let encoded = value.encode_into();
            let (decoded, _) = u32::try_decode_from(&encoded).unwrap();
            black_box(decoded)
        })
    });

    group.bench_function("u64", |b| {
        b.iter(|| {
            let value = black_box(42u64);
            let encoded = value.encode_into();
            let (decoded, _) = u64::try_decode_from(&encoded).unwrap();
            black_box(decoded)
        })
    });

    group.finish();
}

/// Benchmark encoding at different value sizes
fn bench_encoding_value_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_by_size");

    // Test encoding efficiency for different value ranges
    for &size in &[0, 127, 128, 16383, 16384, 65535] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &val| {
            b.iter(|| black_box(val as u32).encode_into())
        });
    }

    group.finish();
}

/// Benchmark sequential encoding/decoding
fn bench_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential");

    group.bench_function("encode_3_values", |b| {
        b.iter(|| {
            let mut buffer = Vec::new();
            buffer.extend_from_slice(&black_box(42u8).encode_into());
            buffer.extend_from_slice(&black_box(1000u16).encode_into());
            buffer.extend_from_slice(&black_box(100000u32).encode_into());
            black_box(buffer)
        })
    });

    // Pre-encode for decode benchmark
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&42u8.encode_into());
    buffer.extend_from_slice(&1000u16.encode_into());
    buffer.extend_from_slice(&100000u32.encode_into());

    group.bench_function("decode_3_values", |b| {
        b.iter(|| {
            let buf = black_box(&buffer);
            let (v1, rest) = u8::try_decode_from(buf).unwrap();
            let (v2, rest) = u16::try_decode_from(rest).unwrap();
            let (v3, _rest) = u32::try_decode_from(rest).unwrap();
            black_box((v1, v2, v3))
        })
    });

    group.finish();
}

/// Benchmark buffer-based encoding operations
fn bench_buffer_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_buffer");

    // u8 buffer encoding
    group.bench_function("u8_small", |b| {
        b.iter(|| {
            let mut buffer = Vec::new();
            black_box(42u8).encode_into_buffer(&mut buffer);
            black_box(buffer)
        })
    });

    // u32 buffer encoding
    group.bench_function("u32_small", |b| {
        b.iter(|| {
            let mut buffer = Vec::new();
            black_box(42u32).encode_into_buffer(&mut buffer);
            black_box(buffer)
        })
    });

    // u64 buffer encoding
    group.bench_function("u64_max", |b| {
        b.iter(|| {
            let mut buffer = Vec::new();
            black_box(u64::MAX).encode_into_buffer(&mut buffer);
            black_box(buffer)
        })
    });

    group.finish();
}

/// Benchmark buffer reuse (most efficient pattern)
fn bench_buffer_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_reuse");

    group.bench_function("sequential_10_values", |b| {
        b.iter(|| {
            let mut buffer = Vec::with_capacity(100);
            for i in 0u8..10 {
                black_box(i).encode_into_buffer(&mut buffer);
            }
            black_box(buffer)
        })
    });

    group.bench_function("mixed_types", |b| {
        b.iter(|| {
            let mut buffer = Vec::with_capacity(100);
            black_box(42u8).encode_into_buffer(&mut buffer);
            black_box(1000u16).encode_into_buffer(&mut buffer);
            black_box(100000u32).encode_into_buffer(&mut buffer);
            black_box(buffer)
        })
    });

    group.finish();
}

/// Benchmark array-based encoding operations
fn bench_array_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_array");

    // u8 array encoding
    group.bench_function("u8_small", |b| {
        b.iter(|| black_box(42u8).encode_into_array())
    });

    group.bench_function("u8_max", |b| {
        b.iter(|| black_box(u8::MAX).encode_into_array())
    });

    // u32 array encoding
    group.bench_function("u32_small", |b| {
        b.iter(|| black_box(42u32).encode_into_array())
    });

    group.bench_function("u32_max", |b| {
        b.iter(|| black_box(u32::MAX).encode_into_array())
    });

    // u64 array encoding
    group.bench_function("u64_small", |b| {
        b.iter(|| black_box(42u64).encode_into_array())
    });

    group.bench_function("u64_max", |b| {
        b.iter(|| black_box(u64::MAX).encode_into_array())
    });

    group.finish();
}

/// Compare all three encoding methods
fn bench_encoding_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoding_comparison");

    // Compare encoding a u32 value using all three methods
    let value = 100000u32;

    group.bench_function("encode_into", |b| b.iter(|| black_box(value).encode_into()));

    group.bench_function("encode_into_buffer", |b| {
        b.iter(|| {
            let mut buffer = Vec::new();
            black_box(value).encode_into_buffer(&mut buffer);
            buffer
        })
    });

    group.bench_function("encode_into_buffer_with_capacity", |b| {
        b.iter(|| {
            let mut buffer = Vec::with_capacity(10);
            black_box(value).encode_into_buffer(&mut buffer);
            buffer
        })
    });

    group.bench_function("encode_into_array", |b| {
        b.iter(|| black_box(value).encode_into_array())
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_encoding,
    bench_decoding,
    bench_roundtrip,
    bench_encoding_value_sizes,
    bench_sequential,
    bench_buffer_encoding,
    bench_buffer_reuse,
    bench_array_encoding,
    bench_encoding_comparison
);
criterion_main!(benches);
