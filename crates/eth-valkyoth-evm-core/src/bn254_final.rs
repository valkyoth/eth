use crate::bn254_tower::Fp12;

const BN254_U: [u64; 1] = [0x44e9_92b4_4a69_09f1];

// Public EIP-197 final exponent `(p^12 - 1) / q` for the BN254 target group.
#[cfg(test)]
const FINAL_EXPONENT: [u64; 48] = [
    0x8696_4b64_ca86_f120,
    0x40a4_efb7_e545_23a4,
    0x837f_a978_96e8_4abb,
    0x3611_02b6_b9b2_b918,
    0xc0de_81de_f356_92da,
    0xbe04_c7e8_a6c3_c760,
    0xd766_f9c9_d570_bb7f,
    0xc230_974d_8356_1841,
    0x5bba_1668_c3be_69a3,
    0x7f38_11c4_1052_6294,
    0x29ba_ee7d_dadd_a71c,
    0xbf81_3b8d_145d_a900,
    0x641b_badf_423f_9a2c,
    0xa80b_b4ea_44ea_cc5e,
    0xcd65_6648_14fd_e37c,
    0x4a03_64b9_5802_91d2,
    0xee93_dfb1_0826_f0dd,
    0x6b42_db8d_c551_4724,
    0xbb10_cf43_0b0f_3785,
    0x4049_4e40_6f80_4216,
    0x55cf_e107_acf3_aafb,
    0x2088_ec80_e0eb_ae87,
    0x846a_3ed0_11a3_37a0,
    0x48a4_5a4a_1e3a_5195,
    0xe566_4568_dfc5_0e16,
    0xab6a_4129_4c0c_c4eb,
    0x82d0_d602_d268_c7da,
    0x6668_449a_ed3c_c48a,
    0x5062_cd0f_b201_5dfc,
    0x7f29_40a8_b1dd_b3d1,
    0x77f5_b63a_2a22_6448,
    0xfef0_7813_61e4_43ae,
    0xf977_870e_88d5_c6c8,
    0x7903_64a6_1f67_6baa,
    0x5887_e72e_cead_dea3,
    0x1377_e563_a09a_1b70,
    0x0c54_efee_1bd8_c3b2,
    0x3ec3_d15a_d524_d8f7,
    0xdaf1_5466_b238_3a5d,
    0xe1e3_0a73_bb94_fec0,
    0x6a1c_7101_5f3f_7be2,
    0x842d_43bf_6369_b1ff,
    0x20fd_dadf_107d_20bc,
    0x0000_002f_4b6d_c970,
    0,
    0,
    0,
    0,
];

pub(crate) fn final_exponentiation(value: Fp12) -> Fp12 {
    let easy = final_exponentiation_easy_part(value);
    final_exponentiation_hard_part(easy)
}

#[cfg(test)]
pub(crate) fn final_exponentiation_reference(value: Fp12) -> Fp12 {
    value.pow_little_endian_limbs(FINAL_EXPONENT)
}

fn final_exponentiation_easy_part(value: Fp12) -> Fp12 {
    let Some(inverse) = value.invert() else {
        return value;
    };
    let p6_over_value = value.frobenius_p6().mul(inverse);
    p6_over_value.mul(p6_over_value.frobenius_p2())
}

fn final_exponentiation_hard_part(value: Fp12) -> Fp12 {
    let fp = value.frobenius();
    let fp2 = value.frobenius_p2();
    let fp3 = fp2.frobenius();

    let fu = value.pow_little_endian_limbs(BN254_U);
    let fu2 = fu.pow_little_endian_limbs(BN254_U);
    let fu3 = fu2.pow_little_endian_limbs(BN254_U);

    let mut y3 = fu.frobenius();
    let fu2p = fu2.frobenius();
    let fu3p = fu3.frobenius();
    let y2 = fu2.frobenius_p2();

    let y0 = fp.mul(fp2).mul(fp3);
    let y1 = value.conjugate();
    let y5 = fu2.conjugate();
    y3 = y3.conjugate();
    let y4 = fu.mul(fu2p).conjugate();
    let y6 = fu3.mul(fu3p).conjugate();

    let mut t0 = y6.square();
    t0 = t0.mul(y4).mul(y5);
    let mut t1 = y3.mul(y5).mul(t0);
    t0 = t0.mul(y2);
    t1 = t1.square().mul(t0).square();
    t0 = t1.mul(y1);
    t1 = t1.mul(y0);
    t0.square().mul(t1)
}
