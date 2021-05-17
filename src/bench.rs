use criterion::{criterion_group, criterion_main, Criterion};
use ff_12381::arith::{W6x64, fe_add, fe_sub, fe_mont_mul, fe_to_mont};
use ff_12381::mont_mul_384_asm;
use num_bigint::BigUint;
use num_traits::Num;
use std::time::Duration;

// RUSTFLAGS="--emit asm" cargo bench
// see, e.g.: target/release/deps/ff_12381-329599c0ba35fa2a.s

// Arbitrary input and expected values (for 1000 iterations, to ensure functionality)
const X: W6x64 = W6x64 {
    v: [0xc34110121829fa85, 0xc42f61586f13abac, 0x5a98f20b2164430a,
        0xcdd6beb839ca6556, 0xdacae65ae941e8e8, 0xf594a44cbdf0ae1]
};

const Y: W6x64 = W6x64 {
    v: [0xf4921aadbbf08d96, 0x9f5973902a56b682, 0x4b86761f89b618b2,
        0xca440e25b9c201dd, 0xd3caeb49dc668726, 0x416ce3c635e5e23]
};

const EXP_SUM: W6x64 = W6x64 {
    v: [0xbd3d31dc0303fa06, 0x704875edd38742a3, 0x60549f5927a1c745,
        0xe234ee37eb7d3cee, 0xa13832d81ab0d5c5, 0x10fdd2f03da8f7ca]
};

const EXP_DIFF: W6x64 = W6x64 {
    v: [0xeb500a9ba3c63dbc, 0xf9d612366c970ad5, 0x581e56b55f02cbcb,
        0x60e49af2737caf46, 0x441baca536704b15, 0xebe95e1d0ff39dc]
};

const EXP_PROD: W6x64 = W6x64 {
    v: [0xb54cf29498954919, 0x8f2491ddb5cef751, 0xb155fe8acce5c7d3,
        0x448683648418e8dd, 0xf3599187e803fc7e, 0x1118bd439ac24052]
};


const MODULUS_STR: &str = "1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab";
const EXPECTED_STR: &str = "169d18ab74c03e6199a9ec1869d2a2a0d53be1749c6acd5028310a17f06383087d69cb203aa01ae0a73a546f5db98555";

// Modular multiplication written strictly with BigUint
fn mul_big(x: &BigUint, y: &BigUint, expected: &BigUint) {  // TODO --> move modulus out
    let mut xx = x.clone();
    let mut yy = y.clone();
    let modulus = BigUint::from_str_radix(MODULUS_STR, 16).unwrap();
    for _i in 0..1_000 {
        let result = (&xx * &yy) % &modulus;
        yy = xx;
        xx = result;
    }
    assert_eq!(&xx, expected)
}

// Montgomery multiplication written in Rust
fn mul_rust(x: &W6x64, y: &W6x64, expected: &W6x64) {
    let mut xx = x.clone();
    let mut yy = y.clone();
    let mut result = W6x64::default();
    for _i in 0..1_000 {
        fe_mont_mul(&mut result, &xx, &yy);
        yy = xx;
        xx = result;
    }
    assert_eq!(&result, expected);
}

// Montgomery addition written in Rust
fn add_rust(x: &W6x64, y: &W6x64, expected: &W6x64) {
    let mut xx = x.clone();
    let mut yy = y.clone();
    let mut result = W6x64::default();
    for _i in 0..1_000 {
        fe_add(&mut result, &xx, &yy);
        yy = xx;
        xx = result;
    }
    assert_eq!(&result, expected);
}

// Montgomery addition written in Rust
fn sub_rust(x: &W6x64, y: &W6x64, expected: &W6x64) {
    let mut xx = x.clone();
    let mut yy = y.clone();
    let mut result = W6x64::default();
    for _i in 0..1_000 {
        fe_sub(&mut result, &xx, &yy);
        yy = xx;
        xx = result;
    }
    assert_eq!(&result, expected);
}


// Montgomery multiplication written in x86-64 assembly
fn mul_asm(x: &W6x64, y: &W6x64, expected: &W6x64) {
    let mut xx = x.clone();
    let mut yy = y.clone();
    let mut result = W6x64::default();
    for _i in 0..1_000 {
        unsafe {
            mont_mul_384_asm(&mut result.v[0], &xx.v[0], &yy.v[0]);
        }
        yy = xx;
        xx = result;
    }
    assert_eq!(&result, expected);
}

// Drive BigUInt multiplication with input and expected result
// Note that BigUInt's are constructed here because Rust won't allow const
pub fn bench_mul_big(c: &mut Criterion) {
    let x = BigUint::from(u128::MAX);
    let y = BigUint::from(u64::MAX);
    let expected = BigUint::from_str_radix(EXPECTED_STR, 16).unwrap();
    c.bench_function("mul_big X 1000 iterations", |b| {
        b.iter(|| mul_big(&x, &y, &expected))
    });
}

// Drive Rust multiplication with input and expected result
pub fn bench_mul_rust(c: &mut Criterion) {
    c.bench_function("Multiplication (Rust) X 1000 iterations", |b| {
        b.iter(|| mul_rust(&X, &Y, &EXP_PROD))
    });
}

// Drive assembly multiplication with input and expected result
pub fn bench_mul_asm(c: &mut Criterion) {
    c.bench_function("mul_asm X 1000 iterations", |b| {
        b.iter(|| mul_asm(&X, &Y, &EXP_PROD))
    });
}

// Drive Rust addition with input and expected result
pub fn bench_add(c: &mut Criterion) {
    c.bench_function("Addition X 1000 iterations", |b| {
        b.iter(|| add_rust(&X, &Y, &EXP_SUM))
    });
}

// Drive Rust subtraction with input and expected result
pub fn bench_sub(c: &mut Criterion) {
    c.bench_function("Subtraction X 1000 iterations", |b| {
        b.iter(|| sub_rust(&X, &Y, &EXP_DIFF))
    });
}

// Run all three benchmarks
criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::new(60, 0));
    targets = bench_add, bench_sub, bench_mul_big, bench_mul_rust, bench_mul_asm
}
criterion_main!(benches);
