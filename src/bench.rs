use criterion::{criterion_group, criterion_main, Criterion};
use ff_12381::arith::{mont_mul_384_rust, W6x64, fe_add, fe_sub};
use ff_12381::mont_mul_384_asm;
use num_bigint::BigUint;
use num_traits::Num;
use std::time::Duration;

// RUSTFLAGS="--emit asm" cargo bench
// see, e.g.: target/release/deps/ff_12381-329599c0ba35fa2a.s

// Arbitrary input and expected values (for 1000 iterations, to ensure functionality)
const X_6X64: W6x64 = W6x64 {
    v: [u64::MAX, u64::MAX, 0, 0, 0, 0],
};
const Y_6X64: W6x64 = W6x64 {
    v: [u64::MAX, 0, 0, 0, 0, 0],
};
const MODULUS_STR: &str = "1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab";
const EXPECTED_STR: &str = "169d18ab74c03e6199a9ec1869d2a2a0d53be1749c6acd5028310a17f06383087d69cb203aa01ae0a73a546f5db98555";
const EXPECTED_W64: W6x64 = W6x64 {
    // Includes R inverse
    v: [
        0xac374deb854388ec,
        0xde27ce2c9adb62a5,
        0xdfa9c2c40422c0ce,
        0x65068c217b3621e3,
        0x4038abbd17ad9397,
        0xa8bcff5b7a351ec,
    ],
};

// Modular multiplication written strictly with BigUint
fn mul_big(x: &BigUint, y: &BigUint, expected: &BigUint) {
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
        mont_mul_384_rust(&mut result, &xx, &yy);
        yy = xx;
        xx = result;
    }
    assert_eq!(&xx, expected);
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
    //assert_eq!(&xx, expected);
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
    //assert_eq!(&xx, expected);
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
    assert_eq!(&xx, expected);
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
    c.bench_function("mul_rust X 1000 iterations", |b| {
        b.iter(|| mul_rust(&X_6X64, &Y_6X64, &EXPECTED_W64))
    });
}

// Drive assembly multiplication with input and expected result
pub fn bench_mul_asm(c: &mut Criterion) {
    c.bench_function("mul_asm X 1000 iterations", |b| {
        b.iter(|| mul_asm(&X_6X64, &Y_6X64, &EXPECTED_W64))
    });
}

// Drive Rust addition with input and expected result
pub fn bench_add(c: &mut Criterion) {
    c.bench_function("add_rust X 1000 iterations", |b| {
        b.iter(|| add_rust(&X_6X64, &Y_6X64, &EXPECTED_W64))
    });
}

// Drive Rust subtraction with input and expected result
pub fn bench_sub(c: &mut Criterion) {
    c.bench_function("sub_rust X 1000 iterations", |b| {
        b.iter(|| sub_rust(&X_6X64, &Y_6X64, &EXPECTED_W64))
    });
}


// Run all three benchmarks
criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::new(10, 0));
    targets = bench_add, bench_sub, bench_mul_big, bench_mul_rust, bench_mul_asm
}
criterion_main!(benches);
