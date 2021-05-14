use std::arch::x86_64::{_addcarry_u64, _subborrow_u64};

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct W6x64 {
    pub v: [u64; 6],  // From least significant [0] to most significant [5]
}

// BLS12-381 field prime, least significant portion first
pub const N: [u64; 6] = [
    0xb9fe_ffff_ffff_aaab, 0x1eab_fffe_b153_ffff, 0x6730_d2a0_f6b0_f624,
    0x6477_4b84_f385_12bf, 0x4b1b_a7b6_434b_acd7, 0x1a01_11ea_397f_e69a,
];

pub fn fe_add(result: &mut W6x64, a: &W6x64, b: &W6x64) {
    let mut sum = W6x64::default();
    let mut carry = false; //0; //false;
    for i in 0..6 {
        let (sum1, carry1) = a.v[i].overflowing_add(b.v[i]);
        let (sum2, carry2) = sum1.overflowing_add(if carry {1} else {0});
        sum.v[i] = sum2;
        carry = carry1 | carry2;
        //unsafe { carry = _addcarry_u64(carry, a.v[i], b.v[i], &mut sum.v[i]) };
    }

    let mut trial = W6x64::default();
    let mut borrow = false; //0; //false;
    for i in 0..6 {
        let (diff, borrow_t) = sum.v[i].overflowing_sub(N[i].wrapping_add(if borrow {1} else {0}));
        trial.v[i] = diff;
        borrow = borrow_t;
        // unsafe { borrow = _subborrow_u64(borrow, sum.v[i], N[i], &mut trial.v[i])}
    }

    let select_sum = u64::from(borrow).wrapping_neg();
    for i in 0..6 {
        result.v[i] = (!select_sum & trial.v[i]) | (select_sum & sum.v[i]);
    }
}

// 2**384 - FIELD_PRIME, least significant portion first
const CORRECTION: [u64; 6] = [
    0x4601_0000_0000_5555, 0xe154_0001_4eac_0000, 0x98cf_2d5f_094f_09db,
    0x9b88_b47b_0c7a_ed40, 0xb4e4_5849_bcb4_5328, 0xe5fe_ee15_c680_1965,
];

pub fn fe_sub(result: &mut W6x64, a: &W6x64, b: &W6x64) {
    let mut diff = W6x64::default();
    let mut borrow_diff = false; //0;
    for i in 0..6 {
        let (diff1, borrow1) = a.v[i].overflowing_sub(b.v[i]);
        let (diff2, borrow2) = diff1.overflowing_sub(if borrow_diff {1} else {0});
        diff.v[i] = diff2;
        borrow_diff = borrow1 | borrow2;
    }

    let mask = u64::from(borrow_diff).wrapping_neg();
    let mut borrow_fix = false; //0;
    for i in 0..6 {
        let (diff1, borrow1) = diff.v[i].overflowing_sub(mask & CORRECTION[i] + if borrow_fix {1} else {0});
        result.v[i] = diff1;
        borrow_fix = borrow1;
    }
}


// R^2 mod N
const R_SQUARED: W6x64 = W6x64 {
    v: [0xf4df_1f34_1c34_1746, 0x0a76_e6a6_09d1_04f1, 0x8de5_476c_4c95_b6d5,
        0x67eb_88a9_939d_83c0, 0x9a79_3e85_b519_952d, 0x1198_8fe5_92ca_e3aa]
};

fn fe_to_montgomery(result: &mut W6x64, a: &W6x64) {
    mont_mul_384_rust(&mut *result, &a, &R_SQUARED);
}

const ONE: W6x64 = W6x64 {
    v: [0x0000_0000_0000_0001, 0x0000_0000_0000_0000, 0x0000_0000_0000_0000,
        0x0000_0000_0000_0000, 0x0000_0000_0000_0000, 0x0000_0000_0000_0000]
};

fn fe_to_normal(result: &mut W6x64, a: &W6x64) {
    mont_mul_384_rust(&mut *result, &a, &ONE);
}


const N_PRIME: u64 = 0x89f3_fffc_fffc_fffd;

pub fn fe_mont_mul(result: &mut W6x64, a: &W6x64, b: &W6x64) {
    let mut temp = [0_u64; 12];

    for i in 0..6 {
        let mut carry = 0_u64;
        for j in 0..6 {
            let hilo = u128::from(a.v[j]) * u128::from(b.v[i]) + u128::from(temp[i + j]) + u128::from(carry);
            temp[i + j] = hilo as u64;
            carry = (hilo >> 64) as u64;
        }
        temp[i + 6] += carry;

        let m: u64 = temp[i].wrapping_mul(N_PRIME);

        let mut carry = 0_u64;
        for j in 0..6 {
            let hilo = u128::from(m) * u128::from(N[j]) + u128::from(temp[i + j]) + u128::from(carry);
            temp[i + j] = hilo as u64; //sum_t1;
            carry = (hilo >> 64) as u64; //((hilo >> 64) as u64) + u64::from(carry_t0) + u64::from(carry_t1);
        }
        temp[i + 6] += carry;
    }

    let mut dec = [0_u64; 6];

    let mut borrow = 0_u64;
    for j in 0..6 {
        let (diff, borrow_t0) = temp[j + 6].overflowing_sub(N[j] + borrow);
        dec[j] = diff as u64;
        borrow = u64::from(borrow_t0); //(diff >> 127) as u64; //u64::from(borrow_t0);
        //let diff = u128::from(temp[j + 6]) - u128::from(N[j] + borrow);
        //dec[j] = diff as u64;
        //borrow = (diff >> 127) as u64;
    }
    let select_temp = borrow.wrapping_neg();
    for j in 0..6 {
        result.v[j] = (select_temp & temp[j + 6]) | (!select_temp & dec[j]);
    }
}





#[allow(clippy::cast_possible_truncation)] // for hilo truncation as u64
pub fn mont_mul_384_rust(result: &mut W6x64, a: &W6x64, b: &W6x64) {
    let mut temp = [0_u64; 12];

    for i in 0..6 {
        let mut carry = 0_u64;
        for j in 0..6 {
            let hilo = u128::from(a.v[j]) * u128::from(b.v[i]);
            let (sum_t0, carry_t0) = (hilo as u64).overflowing_add(temp[i + j]);
            let (sum_t1, carry_t1) = sum_t0.overflowing_add(carry);
            temp[i + j] = sum_t1;
            carry = ((hilo >> 64) as u64) + u64::from(carry_t0) + u64::from(carry_t1);
        }
        temp[i + 6] += carry;

        let m: u64 = temp[i].wrapping_mul(N_PRIME);

        let mut carry = 0_u64;
        for j in 0..6 {
            let hilo = u128::from(m) * u128::from(N[j]);
            let (sum_t0, carry_t0) = (hilo as u64).overflowing_add(temp[i + j]);
            let (sum_t1, carry_t1) = sum_t0.overflowing_add(carry);
            temp[i + j] = sum_t1;
            carry = ((hilo >> 64) as u64) + u64::from(carry_t0) + u64::from(carry_t1);
        }
        temp[i + 6] += carry;
    }

    let mut dec = [0_u64; 6];

    let mut borrow = 0_u64;
    for j in 0..6 {
        let (diff, borrow_t0) = temp[j + 6].overflowing_sub(N[j] + borrow);
        dec[j] = diff;
        borrow = u64::from(borrow_t0);
    }
    let select_temp = borrow.wrapping_neg();
    for j in 0..6 {
        result.v[j] = (select_temp & temp[j + 6]) | (!select_temp & dec[j]);
    }
}
