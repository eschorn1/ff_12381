#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::cargo)]

pub mod arith;

extern "C" {
    pub fn mont_mul_384_asm(result: &mut u64, a: &u64, b: &u64);
}

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod tests {
    use crate::arith::{mont_mul_384_rust, W6x64};
    use crate::mont_mul_384_asm;
    use num_bigint::BigUint;
    use num_traits::Num;
    use rand::Rng;
    use std::convert::TryInto;

    lazy_static! {
        static ref MODULUS_BIG: BigUint = BigUint::from_str_radix(
            "1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab",
            16
        ).unwrap();
        static ref R: BigUint = BigUint::from(1_u8) << 384;
    }

    fn rnd_big_mod_n() -> BigUint {
        let mut rnd_bytes = [0_u8; 64];
        rand::thread_rng().fill(&mut rnd_bytes[..]);
        BigUint::from_bytes_le(&rnd_bytes) % &(*MODULUS_BIG)
    }

    fn big_to_w6x64_r(x: &BigUint) -> W6x64 {
        let x_r: BigUint = (x * &(*R)) % &(*MODULUS_BIG);
        let mut bytes = [0_u8; 48];
        bytes[0..(((7 + x_r.bits()) / 8) as usize)].clone_from_slice(&x_r.to_bytes_le());
        let mut result = W6x64::default();
        for i in 0..6 {
            result.v[i] = u64::from_le_bytes(bytes[i * 8..(i + 1) * 8].try_into().unwrap());
        }
        result
    }

    #[test]
    fn test_mont_mul_384() {
        let mut actual_asm = W6x64::default();
        let mut actual_rust = W6x64::default();

        // 2B -> 5H
        for _i in 0..5_000_000 {
            let a_big = rnd_big_mod_n();
            let b_big = rnd_big_mod_n();
            let a_w64 = big_to_w6x64_r(&a_big);
            let b_w64 = big_to_w6x64_r(&b_big);

            let expected = (&a_big * &b_big) % &(*MODULUS_BIG);
            unsafe {
                mont_mul_384_asm(&mut actual_asm.v[0], &a_w64.v[0], &b_w64.v[0]);
            };
            mont_mul_384_rust(&mut actual_rust, &a_w64, &b_w64);

            assert_eq!(big_to_w6x64_r(&expected), actual_asm);
            assert_eq!(actual_asm, actual_rust);
        }
    }
}
