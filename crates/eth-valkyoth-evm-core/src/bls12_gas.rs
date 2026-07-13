use crate::{EvmCoreError, EvmGas};

const G1_ITEM_BYTES: usize = 160;
const G2_ITEM_BYTES: usize = 288;
const PAIRING_ITEM_BYTES: usize = 384;
const DISCOUNT_DIVISOR: u64 = 1_000;
const G1_MUL_GAS: u64 = 12_000;
const G2_MUL_GAS: u64 = 22_500;

#[rustfmt::skip]
const G1_DISCOUNTS: [u16; 128] = [
    1000, 949, 848, 797, 764, 750, 738, 728, 719, 712, 705, 698, 692, 687, 682, 677,
    673, 669, 665, 661, 658, 654, 651, 648, 645, 642, 640, 637, 635, 632, 630, 627,
    625, 623, 621, 619, 617, 615, 613, 611, 609, 608, 606, 604, 603, 601, 599, 598,
    596, 595, 593, 592, 591, 589, 588, 586, 585, 584, 582, 581, 580, 579, 577, 576,
    575, 574, 573, 572, 570, 569, 568, 567, 566, 565, 564, 563, 562, 561, 560, 559,
    558, 557, 556, 555, 554, 553, 552, 551, 550, 549, 548, 547, 547, 546, 545, 544,
    543, 542, 541, 540, 540, 539, 538, 537, 536, 536, 535, 534, 533, 532, 532, 531,
    530, 529, 528, 528, 527, 526, 525, 525, 524, 523, 522, 522, 521, 520, 520, 519,
];

#[rustfmt::skip]
const G2_DISCOUNTS: [u16; 128] = [
    1000, 1000, 923, 884, 855, 832, 812, 796, 782, 770, 759, 749, 740, 732, 724, 717,
    711, 704, 699, 693, 688, 683, 679, 674, 670, 666, 663, 659, 655, 652, 649, 646,
    643, 640, 637, 634, 632, 629, 627, 624, 622, 620, 618, 615, 613, 611, 609, 607,
    606, 604, 602, 600, 598, 597, 595, 593, 592, 590, 589, 587, 586, 584, 583, 582,
    580, 579, 578, 576, 575, 574, 573, 571, 570, 569, 568, 567, 566, 565, 563, 562,
    561, 560, 559, 558, 557, 556, 555, 554, 553, 552, 552, 551, 550, 549, 548, 547,
    546, 545, 545, 544, 543, 542, 541, 541, 540, 539, 538, 537, 537, 536, 535, 535,
    534, 533, 532, 532, 531, 530, 530, 529, 528, 528, 527, 526, 526, 525, 524, 524,
];

pub(crate) fn g1_msm(input_len: usize) -> Result<EvmGas, EvmCoreError> {
    msm(input_len, G1_ITEM_BYTES, G1_MUL_GAS, &G1_DISCOUNTS)
}

pub(crate) fn g2_msm(input_len: usize) -> Result<EvmGas, EvmCoreError> {
    msm(input_len, G2_ITEM_BYTES, G2_MUL_GAS, &G2_DISCOUNTS)
}

pub(crate) fn pairing(input_len: usize) -> Result<EvmGas, EvmCoreError> {
    let pairs = item_count(input_len, PAIRING_ITEM_BYTES)?;
    let variable = pairs
        .checked_mul(32_600)
        .ok_or(EvmCoreError::PrecompileGasOverflow)?;
    let total = variable
        .checked_add(37_700)
        .ok_or(EvmCoreError::PrecompileGasOverflow)?;
    Ok(EvmGas::new(total))
}

fn msm(
    input_len: usize,
    item_bytes: usize,
    multiplication_gas: u64,
    discounts: &[u16; 128],
) -> Result<EvmGas, EvmCoreError> {
    let items = item_count(input_len, item_bytes)?;
    let discount = discount_for(items, discounts);
    let full_price = items
        .checked_mul(multiplication_gas)
        .ok_or(EvmCoreError::PrecompileGasOverflow)?;
    let discounted = full_price
        .checked_mul(discount)
        .ok_or(EvmCoreError::PrecompileGasOverflow)?;
    Ok(EvmGas::new(discounted / DISCOUNT_DIVISOR))
}

fn item_count(input_len: usize, item_bytes: usize) -> Result<u64, EvmCoreError> {
    let items = input_len
        .checked_div(item_bytes)
        .ok_or(EvmCoreError::PrecompileGasOverflow)?;
    u64::try_from(items).map_err(|_| EvmCoreError::PrecompileGasOverflow)
}

fn discount_for(items: u64, discounts: &[u16; 128]) -> u64 {
    let index = usize::try_from(items.saturating_sub(1)).unwrap_or(usize::MAX);
    let capped = discounts.last().copied().unwrap_or(0);
    let discount = discounts.get(index).copied().unwrap_or(capped);
    u64::from(discount)
}
