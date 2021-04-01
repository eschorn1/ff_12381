use criterion::{criterion_group, criterion_main, Criterion};
use ff12381::arith::{fe_mul, W64};
use ff12381::mont_mul_384_asm;
use num_bigint::BigUint;
use num_traits::Num;
use std::time::Duration;

// Arbitrary input and expected values (to ensure functionality)
const X_64: W64 = W64 {
    v: [u64::MAX, u64::MAX, 0, 0, 0, 0],
};
const Y_64: W64 = W64 {
    v: [u64::MAX, 0, 0, 0, 0, 0],
};
const MODULUS_STR: &str = "1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab";
const MODULUS_W64: W64 = W64 {
    v: [
        0xb9fe_ffff_ffff_aaab,
        0x1eab_fffe_b153_ffff,
        0x6730_d2a0_f6b0_f624,
        0x6477_4b84_f385_12bf,
        0x4b1b_a7b6_434b_acd7,
        0x1a01_11ea_397f_e69a,
    ],
};
const EXPECTED_STR: &str = "169d18ab74c03e6199a9ec1869d2a2a0d53be1749c6acd5028310a17f06383087d69cb203aa01ae0a73a546f5db98555";
const EXPECTED_W64: W64 = W64 {
    v: [
        0xac374deb854388ec,
        0xde27ce2c9adb62a5,
        0xdfa9c2c40422c0ce,
        0x65068c217b3621e3,
        0x4038abbd17ad9397,
        0xa8bcff5b7a351ec,
    ],
};

// Modular multiplication written strictly with BigUints
fn fe_mul_big(x: &BigUint, y: &BigUint, modulus: &BigUint, expected: &BigUint) {
    let mut xx = x.clone();
    let mut yy = y.clone();
    for _i in 0..1_000 {
        let result = (&xx * &yy) % modulus;
        yy = xx;
        xx = result;
    }
    assert_eq!(&xx, expected, "xx was {:x}", xx)
}

// Montgomery multiplication written in Rust
fn fe_mul_rust(x: &W64, y: &W64, modulus: &W64, expected: &W64) {
    let mut xx = x.clone();
    let mut yy = y.clone();
    let mut result = W64::default();

    for _i in 0..1_000 {
        fe_mul(&mut result, &xx, &yy, &modulus);
        yy = xx;
        xx = result;
    }
    assert_eq!(&xx, expected, "xx was {:x?}", xx);
}

// Montgomery multiplication written in x86-64 assembly
fn fe_mul_asm(x: &W64, y: &W64, modulus: &W64, expected: &W64) {
    let mut xx = x.clone();
    let mut yy = y.clone();
    let mut result = W64::default();

    for _i in 0..1_000 {
        unsafe {
            mont_mul_384_asm(&mut result.v[0], &xx.v[0], &yy.v[0], &modulus.v[0]);
        }
        yy = xx;
        xx = result;
    }
    assert_eq!(&xx, expected, "xx was {:x?}", xx);
}

// Drive BigUInt multiplication with input and expected result
// Note that BigUInt's are constructed here because Rust won't allow const
pub fn bench_fe_mul_big(c: &mut Criterion) {
    let x = BigUint::from(u128::MAX);
    let y = BigUint::from(u64::MAX);
    let modulus: BigUint = BigUint::from_str_radix(MODULUS_STR, 16).unwrap();
    let expected = BigUint::from_str_radix(EXPECTED_STR, 16).unwrap();
    c.bench_function("fe_mul_big X 1000 iterations", |b| {
        b.iter(|| fe_mul_big(&x, &y, &modulus, &expected))
    });
}

// Drive Rust multiplication with input and expected result
pub fn bench_fe_mul_rust(c: &mut Criterion) {
    c.bench_function("fe_mul_rust X 1000 iterations", |b| {
        b.iter(|| fe_mul_rust(&X_64, &Y_64, &MODULUS_W64, &EXPECTED_W64))
    });
}

// Drive assembly multiplication with input and expected result
pub fn bench_fe_mul_asm(c: &mut Criterion) {
    c.bench_function("fe_mul_asm X 1000 iterations", |b| {
        b.iter(|| fe_mul_asm(&X_64, &Y_64, &MODULUS_W64, &EXPECTED_W64))
    });
}

// Run all three benchmarks
criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::new(20, 0));
    targets = bench_fe_mul_big, bench_fe_mul_rust, bench_fe_mul_asm
}
criterion_main!(benches);
