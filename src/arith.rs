#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq)] // TODO: Implement CT equal
pub struct W64 {
    pub v: [u64; 6],
}

const N_PRIME: u64 = 0x89f3_fffc_fffc_fffd;

pub fn fe_mul(result: &mut W64, a: &W64, b: &W64, modulus: &W64) {
    let mut temp = [0_u64; 13];

    for i in 0..6 {
        let mut carry = 0_u64;
        for j in 0..6 {
            let hilo = u128::from(a.v[j]) * u128::from(b.v[i]);
            let (sum_t0, carry_t0) = (hilo as u64).overflowing_add(temp[i + j]);
            let (sum_t1, carry_t1) = sum_t0.overflowing_add(carry);
            temp[i + j] = sum_t1;
            carry = ((hilo >> 64) as u64) + (if carry_t0 { 1 } else { 0 }) + (if carry_t1 { 1 } else { 0 });
        }
        let sum = temp[i + 6] + carry;
        temp[i + 6] = sum;

        let m: u64 = temp[i].wrapping_mul(N_PRIME);

        let mut carry = 0_u64;
        for j in 0..6 {
            let hilo = u128::from(m) * u128::from(modulus.v[j]);
            let (sum_t0, carry_t0) = (hilo as u64).overflowing_add(temp[i + j]);
            let (sum_t1, carry_t1) = sum_t0.overflowing_add(carry);
            temp[i + j] = sum_t1;
            carry = ((hilo >> 64) as u64) + (if carry_t0 { 1 } else { 0 }) + (if carry_t1 { 1 } else { 0 });
        }
        let sum = temp[i + 6] + carry;
        temp[i + 6] = sum;
    }

    let mut dec = [0_u64; 6];

    let mut borrow = 0;
    for j in 0..6 {
        let (diff, borrow_t0) = temp[j+6].overflowing_sub(modulus.v[j] + borrow);
        dec[j] = diff as u64;
        borrow = borrow_t0 as u64;
    }
    let under = u64::from(borrow).wrapping_neg();
    for j in 0..6 {
        result.v[j] = (under & temp[j + 6]) | (!under & dec[j]);
    }
}
