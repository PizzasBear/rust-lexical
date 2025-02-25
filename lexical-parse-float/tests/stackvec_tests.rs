mod stackvec;

use core::cmp;

use lexical_parse_float::bigint::{self, Limb, StackVec, LIMB_BITS};
use stackvec::vec_from_u32;

const SIZE: usize = 50;
type VecType = StackVec<SIZE>;

#[test]
fn simple_test() {
    // Test the simple properties of the stack vector.
    let mut x = VecType::from_u32(1);
    assert_eq!(x.len(), 1);
    assert_eq!(x.is_empty(), false);
    assert_eq!(x.capacity(), SIZE);
    x.try_push(5).unwrap();
    assert_eq!(x.len(), 2);
    assert_eq!(x.pop(), Some(5));
    assert_eq!(x.len(), 1);
    assert_eq!(&*x, &[1]);
    x.try_extend(&[2, 3, 4]).unwrap();
    assert_eq!(x.len(), 4);
    assert_eq!(&*x, &[1, 2, 3, 4]);
    x.try_resize(6, 0).unwrap();
    assert_eq!(x.len(), 6);
    assert_eq!(&*x, &[1, 2, 3, 4, 0, 0]);
    x.try_resize(0, 0).unwrap();
    assert_eq!(x.len(), 0);
    assert_eq!(x.is_empty(), true);

    let x = VecType::try_from(&[5, 1]).unwrap();
    assert_eq!(x.len(), 2);
    assert_eq!(x.is_empty(), false);
    assert_eq!(x.hi16(), (0x8000, true));
    if LIMB_BITS == 32 {
        assert_eq!(x.hi32(), (0x80000002, true));
        assert_eq!(x.hi64(), (0x8000000280000000, false));
    } else {
        assert_eq!(x.hi32(), (0x80000000, true));
        assert_eq!(x.hi64(), (0x8000000000000002, true));
    }
    let rview = x.rview();
    assert_eq!(x[0], 5);
    assert_eq!(x[1], 1);
    assert_eq!(rview[0], 1);
    assert_eq!(rview[1], 5);
    assert_eq!(rview.get(1), Some(&5));
    assert_eq!(rview.get(2), None);

    assert_eq!(VecType::from_u16(u16::MAX).hi16(), (u16::MAX, false));
    assert_eq!(VecType::from_u32(u32::MAX).hi32(), (u32::MAX, false));
    assert_eq!(VecType::from_u64(u64::MAX).hi64(), (u64::MAX, false));
}

#[test]
fn bounds_test() {
    type ShortVec = StackVec<2>;
    let mut x = ShortVec::from_u32(1);
    assert_eq!(x.try_push(2), Some(()));
    assert_eq!(x.try_push(5), None);

    assert_eq!(x.try_resize(0, 0), Some(()));
    assert_eq!(x.try_resize(3, 0), None);
}

#[test]
fn hi32_test() {
    assert_eq!(VecType::from_u16(0xA).hi32(), (0xA0000000, false));
    assert_eq!(VecType::from_u32(0xAB).hi32(), (0xAB000000, false));
    assert_eq!(VecType::from_u64(0xAB00000000).hi32(), (0xAB000000, false));
    assert_eq!(VecType::from_u64(0xA23456789A).hi32(), (0xA2345678, true));
}

#[test]
fn hi64_test() {
    assert_eq!(VecType::from_u16(0xA).hi64(), (0xA000000000000000, false));
    assert_eq!(VecType::from_u32(0xAB).hi64(), (0xAB00000000000000, false));
    assert_eq!(VecType::from_u64(0xAB00000000).hi64(), (0xAB00000000000000, false));
    assert_eq!(VecType::from_u64(0xA23456789A).hi64(), (0xA23456789A000000, false));
}

#[test]
fn cmp_test() {
    // Simple
    let x = VecType::from_u32(1);
    let y = VecType::from_u32(2);
    assert_eq!(x.partial_cmp(&x), Some(cmp::Ordering::Equal));
    assert_eq!(x.cmp(&x), cmp::Ordering::Equal);
    assert_eq!(x.cmp(&y), cmp::Ordering::Less);

    // Check asymmetric
    let x = VecType::try_from(&[5, 1]).unwrap();
    let y = VecType::from_u32(2);
    assert_eq!(x.cmp(&x), cmp::Ordering::Equal);
    assert_eq!(x.cmp(&y), cmp::Ordering::Greater);

    // Check when we use reverse ordering properly.
    let x = VecType::try_from(&[5, 1, 9]).unwrap();
    let y = VecType::try_from(&[6, 2, 8]).unwrap();
    assert_eq!(x.cmp(&x), cmp::Ordering::Equal);
    assert_eq!(x.cmp(&y), cmp::Ordering::Greater);

    // Complex scenario, check it properly uses reverse ordering.
    let x = VecType::try_from(&[0, 1, 9]).unwrap();
    let y = VecType::try_from(&[4294967295, 0, 9]).unwrap();
    assert_eq!(x.cmp(&x), cmp::Ordering::Equal);
    assert_eq!(x.cmp(&y), cmp::Ordering::Greater);
}

#[test]
fn math_test() {
    let mut x = VecType::try_from(&[0, 1, 9]).unwrap();
    assert_eq!(x.is_normalized(), true);
    x.try_push(0).unwrap();
    assert_eq!(&*x, &[0, 1, 9, 0]);
    assert_eq!(x.is_normalized(), false);
    x.normalize();
    assert_eq!(&*x, &[0, 1, 9]);
    assert_eq!(x.is_normalized(), true);

    x.add_small(1);
    assert_eq!(&*x, &[1, 1, 9]);
    x.add_small(Limb::MAX);
    assert_eq!(&*x, &[0, 2, 9]);

    x.mul_small(3);
    assert_eq!(&*x, &[0, 6, 27]);
    x.mul_small(Limb::MAX);
    let expected: VecType = if LIMB_BITS == 32 {
        vec_from_u32(&[0, 4294967290, 4294967274, 26])
    } else {
        vec_from_u32(&[0, 0, 4294967290, 4294967295, 4294967274, 4294967295, 26])
    };
    assert_eq!(&*x, &*expected);

    #[cfg(feature = "radix")]
    {
        let mut x: VecType = vec_from_u32(&[0, 0, 0, 536870912]);
        let y: VecType = vec_from_u32(&[3358091099, 2770363594, 2782716766, 217327764]);
        assert_eq!(x.quorem(&y), 2);
        let expected: VecType = vec_from_u32(&[1873752394, 3049207402, 3024501058, 102215382]);
        assert_eq!(&*x, &*expected);
    }

    let mut x = VecType::from_u32(0xFFFFFFFF);
    let y = VecType::from_u32(5);
    x *= &y;
    let expected: VecType = vec_from_u32(&[0xFFFFFFFB, 0x4]);
    assert_eq!(&*x, &*expected);

    // Test with carry
    let mut x = VecType::from_u32(1);
    assert_eq!(&*x, &[1]);
    x.add_small(Limb::MAX);
    assert_eq!(&*x, &[0, 1]);
}

#[test]
fn hi_test() {
    assert_eq!(unsafe { bigint::nonzero(&[0, 0, 0], 0) }, false);
    assert_eq!(unsafe { bigint::nonzero(&[1, 0, 0], 0) }, true);

    assert_eq!(bigint::u32_to_hi16_1(1), (0x8000, false));
    assert_eq!(bigint::u32_to_hi16_2(1, 4), (0x8000, true));
    assert_eq!(bigint::u32_to_hi32_1(1), (0x80000000, false));
    assert_eq!(bigint::u32_to_hi32_2(1, 4), (0x80000002, false));
    assert_eq!(bigint::u32_to_hi64_1(1), (0x8000000000000000, false));
    assert_eq!(bigint::u32_to_hi64_2(1, 4), (0x8000000200000000, false));
    assert_eq!(bigint::u32_to_hi64_2(1, 5), (0x8000000280000000, false));
    assert_eq!(bigint::u32_to_hi64_3(1, 5, 4), (0x8000000280000002, false));
    assert_eq!(bigint::u32_to_hi64_3(1, 5, 5), (0x8000000280000002, true));

    assert_eq!(bigint::u64_to_hi16_1(1), (0x8000, false));
    assert_eq!(bigint::u64_to_hi16_2(1, 4), (0x8000, true));
    assert_eq!(bigint::u64_to_hi32_1(1), (0x80000000, false));
    assert_eq!(bigint::u64_to_hi32_2(1, 4), (0x80000000, true));
    assert_eq!(bigint::u64_to_hi64_1(1), (0x8000000000000000, false));
    assert_eq!(bigint::u64_to_hi64_2(1, 4), (0x8000000000000002, false));
    assert_eq!(bigint::u64_to_hi64_2(1, 5), (0x8000000000000002, true));
}

#[test]
fn scalar_add_test() {
    assert_eq!(bigint::scalar_add(5, 5), (10, false));
    assert_eq!(bigint::scalar_add(Limb::MAX, 1), (0, true));
}

#[test]
fn scalar_mul_test() {
    assert_eq!(bigint::scalar_mul(5, 5, 0), (25, 0));
    assert_eq!(bigint::scalar_mul(5, 5, 1), (26, 0));
    assert_eq!(bigint::scalar_mul(Limb::MAX, 2, 0), (Limb::MAX - 1, 1));
}

#[test]
fn small_add_test() {
    let mut x = VecType::from_u32(4294967295);
    bigint::small_add(&mut x, 5);
    let expected: VecType = vec_from_u32(&[4, 1]);
    assert_eq!(&*x, &*expected);

    let mut x = VecType::from_u32(5);
    bigint::small_add(&mut x, 7);
    let expected = VecType::from_u32(12);
    assert_eq!(&*x, &*expected);

    // Single carry, internal overflow
    let mut x = VecType::from_u64(0x80000000FFFFFFFF);
    bigint::small_add(&mut x, 7);
    let expected: VecType = vec_from_u32(&[6, 0x80000001]);
    assert_eq!(&*x, &*expected);

    // Double carry, overflow
    let mut x = VecType::from_u64(0xFFFFFFFFFFFFFFFF);
    bigint::small_add(&mut x, 7);
    let expected: VecType = vec_from_u32(&[6, 0, 1]);
    assert_eq!(&*x, &*expected);
}

#[test]
fn small_mul_test() {
    // No overflow check, 1-int.
    let mut x = VecType::from_u32(5);
    bigint::small_mul(&mut x, 7);
    let expected = VecType::from_u32(35);
    assert_eq!(&*x, &*expected);

    // No overflow check, 2-ints.
    let mut x = VecType::from_u64(0x4000000040000);
    bigint::small_mul(&mut x, 5);
    let expected: VecType = vec_from_u32(&[0x00140000, 0x140000]);
    assert_eq!(&*x, &*expected);

    // Overflow, 1 carry.
    let mut x = VecType::from_u32(0x33333334);
    bigint::small_mul(&mut x, 5);
    let expected: VecType = vec_from_u32(&[4, 1]);
    assert_eq!(&*x, &*expected);

    // Overflow, 1 carry, internal.
    let mut x = VecType::from_u64(0x133333334);
    bigint::small_mul(&mut x, 5);
    let expected: VecType = vec_from_u32(&[4, 6]);
    assert_eq!(&*x, &*expected);

    // Overflow, 2 carries.
    let mut x = VecType::from_u64(0x3333333333333334);
    bigint::small_mul(&mut x, 5);
    let expected: VecType = vec_from_u32(&[4, 0, 1]);
    assert_eq!(&*x, &*expected);
}

#[test]
fn pow_test() {
    let mut x = VecType::from_u32(1);
    bigint::pow(&mut x, 5, 2);
    let expected = VecType::from_u32(25);
    assert_eq!(&*x, &*expected);

    let mut x = VecType::from_u32(1);
    bigint::pow(&mut x, 5, 15);
    let expected: VecType = vec_from_u32(&[452807053, 7]);
    assert_eq!(&*x, &*expected);

    let mut x = VecType::from_u32(1);
    bigint::pow(&mut x, 5, 16);
    let expected: VecType = vec_from_u32(&[2264035265, 35]);
    assert_eq!(&*x, &*expected);

    let mut x = VecType::from_u32(1);
    bigint::pow(&mut x, 5, 17);
    let expected: VecType = vec_from_u32(&[2730241733, 177]);
    assert_eq!(&*x, &*expected);

    let mut x = VecType::from_u32(1);
    bigint::pow(&mut x, 5, 302);
    let expected: VecType = vec_from_u32(&[
        2443090281, 2149694430, 2297493928, 1584384001, 1279504719, 1930002239, 3312868939,
        3735173465, 3523274756, 2025818732, 1641675015, 2431239749, 4292780461, 3719612855,
        4174476133, 3296847770, 2677357556, 638848153, 2198928114, 3285049351, 2159526706,
        626302612,
    ]);
    assert_eq!(&*x, &*expected);
}

#[test]
fn large_add_test() {
    // Overflow, both single values
    let mut x = VecType::from_u32(4294967295);
    let y = VecType::from_u32(5);
    bigint::large_add(&mut x, &y);
    let expected: VecType = vec_from_u32(&[4, 1]);
    assert_eq!(&*x, &*expected);

    // No overflow, single value
    let mut x = VecType::from_u32(5);
    let y = VecType::from_u32(7);
    bigint::large_add(&mut x, &y);
    let expected = VecType::from_u32(12);
    assert_eq!(&*x, &*expected);

    // Single carry, internal overflow
    let mut x = VecType::from_u64(0x80000000FFFFFFFF);
    let y = VecType::from_u32(7);
    bigint::large_add(&mut x, &y);
    let expected: VecType = vec_from_u32(&[6, 0x80000001]);
    assert_eq!(&*x, &*expected);

    // 1st overflows, 2nd doesn't.
    let mut x = VecType::from_u64(0x7FFFFFFFFFFFFFFF);
    let y = VecType::from_u64(0x7FFFFFFFFFFFFFFF);
    bigint::large_add(&mut x, &y);
    let expected: VecType = vec_from_u32(&[0xFFFFFFFE, 0xFFFFFFFF]);
    assert_eq!(&*x, &*expected);

    // Both overflow.
    let mut x = VecType::from_u64(0x8FFFFFFFFFFFFFFF);
    let y = VecType::from_u64(0x7FFFFFFFFFFFFFFF);
    bigint::large_add(&mut x, &y);
    let expected: VecType = vec_from_u32(&[0xFFFFFFFE, 0x0FFFFFFF, 1]);
    assert_eq!(&*x, &*expected);
}

#[test]
fn large_mul_test() {
    // Test by empty
    let mut x = VecType::from_u32(0xFFFFFFFF);
    let y = VecType::new();
    bigint::large_mul(&mut x, &y);
    let expected = VecType::new();
    assert_eq!(&*x, &*expected);

    // Simple case
    let mut x = VecType::from_u32(0xFFFFFFFF);
    let y = VecType::from_u32(5);
    bigint::large_mul(&mut x, &y);
    let expected: VecType = vec_from_u32(&[0xFFFFFFFB, 0x4]);
    assert_eq!(&*x, &*expected);

    // Large u32, but still just as easy.
    let mut x = VecType::from_u32(0xFFFFFFFF);
    let y = VecType::from_u32(0xFFFFFFFE);
    bigint::large_mul(&mut x, &y);
    let expected: VecType = vec_from_u32(&[0x2, 0xFFFFFFFD]);
    assert_eq!(&*x, &*expected);

    // Let's multiply two large values together.
    let mut x: VecType = vec_from_u32(&[0xFFFFFFFE, 0x0FFFFFFF, 1]);
    let y: VecType = vec_from_u32(&[0x99999999, 0x99999999, 0xCCCD9999, 0xCCCC]);
    bigint::large_mul(&mut x, &y);
    let expected: VecType =
        vec_from_u32(&[0xCCCCCCCE, 0x5CCCCCCC, 0x9997FFFF, 0x33319999, 0x999A7333, 0xD999]);
    assert_eq!(&*x, &*expected);
}

#[test]
fn very_large_mul_test() {
    // Test cases triggered to that would normally use `karatsuba_mul`.
    // Karatsuba multiplication was ripped out, however, these are useful
    // test cases.
    let mut x: VecType = vec_from_u32(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let y: VecType = vec_from_u32(&[4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]);
    bigint::large_mul(&mut x, &y);
    let expected: VecType = vec_from_u32(&[
        4, 13, 28, 50, 80, 119, 168, 228, 300, 385, 484, 598, 728, 875, 1040, 1224, 1340, 1435,
        1508, 1558, 1584, 1585, 1560, 1508, 1428, 1319, 1180, 1010, 808, 573, 304,
    ]);
    assert_eq!(&*x, &*expected);

    // Test cases triggered to that would normally use `karatsuba_uneven_mul`.
    let mut x: VecType = vec_from_u32(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let y: VecType = vec_from_u32(&[
        4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
        28, 29, 30, 31, 32, 33, 34, 35, 36, 37,
    ]);
    bigint::large_mul(&mut x, &y);
    let expected: VecType = vec_from_u32(&[
        4, 13, 28, 50, 80, 119, 168, 228, 300, 385, 484, 598, 728, 875, 1040, 1224, 1360, 1496,
        1632, 1768, 1904, 2040, 2176, 2312, 2448, 2584, 2720, 2856, 2992, 3128, 3264, 3400, 3536,
        3672, 3770, 3829, 3848, 3826, 3762, 3655, 3504, 3308, 3066, 2777, 2440, 2054, 1618, 1131,
        592,
    ]);
    assert_eq!(&*x, &*expected);
}

#[test]
#[cfg(feature = "radix")]
fn quorem_test() {
    let mut x: VecType = vec_from_u32(&[0, 0, 0, 536870912]);
    let y: VecType = vec_from_u32(&[3358091099, 2770363594, 2782716766, 217327764]);
    assert_eq!(bigint::large_quorem(&mut x, &y), 2);
    let expected: VecType = vec_from_u32(&[1873752394, 3049207402, 3024501058, 102215382]);
    assert_eq!(&*x, &*expected);
}

#[test]
fn bit_length_test() {
    let x: VecType = vec_from_u32(&[0, 0, 0, 1]);
    assert_eq!(bigint::bit_length(&x), 97);

    let x: VecType = vec_from_u32(&[0, 0, 0, 3]);
    assert_eq!(bigint::bit_length(&x), 98);

    let x = VecType::from_u32(1 << 31);
    assert_eq!(bigint::bit_length(&x), 32);
}

#[test]
fn shl_bits_test() {
    let mut x = VecType::from_u32(0xD2210408);
    bigint::shl_bits(&mut x, 5);
    let expected: VecType = vec_from_u32(&[0x44208100, 0x1A]);
    assert_eq!(&*x, &*expected);
}

#[test]
fn shl_limbs_test() {
    let mut x = VecType::from_u32(0xD2210408);
    bigint::shl_limbs(&mut x, 2);
    let expected: VecType = if LIMB_BITS == 32 {
        vec_from_u32(&[0, 0, 0xD2210408])
    } else {
        vec_from_u32(&[0, 0, 0, 0, 0xD2210408])
    };
    assert_eq!(&*x, &*expected);
}

#[test]
fn shl_test() {
    // Pattern generated via `''.join(["1" +"0"*i for i in range(20)])`
    let mut x = VecType::from_u32(0xD2210408);
    bigint::shl(&mut x, 5).unwrap();
    let expected: VecType = vec_from_u32(&[0x44208100, 0x1A]);
    assert_eq!(&*x, &*expected);

    bigint::shl(&mut x, 32).unwrap();
    let expected: VecType = vec_from_u32(&[0, 0x44208100, 0x1A]);
    assert_eq!(&*x, &*expected);

    bigint::shl(&mut x, 27).unwrap();
    let expected: VecType = vec_from_u32(&[0, 0, 0xD2210408]);
    assert_eq!(&*x, &*expected);

    // 96-bits of previous pattern
    let mut x: VecType = vec_from_u32(&[0x20020010, 0x8040100, 0xD2210408]);
    bigint::shl(&mut x, 5).unwrap();
    let expected: VecType = vec_from_u32(&[0x400200, 0x802004, 0x44208101, 0x1A]);
    assert_eq!(&*x, &*expected);

    bigint::shl(&mut x, 32).unwrap();
    let expected: VecType = vec_from_u32(&[0, 0x400200, 0x802004, 0x44208101, 0x1A]);
    assert_eq!(&*x, &*expected);

    bigint::shl(&mut x, 27).unwrap();
    let expected: VecType = vec_from_u32(&[0, 0, 0x20020010, 0x8040100, 0xD2210408]);
    assert_eq!(&*x, &*expected);
}

#[test]
fn split_radix_test() {
    assert_eq!(bigint::split_radix(10), (5, 1));
    if cfg!(feature = "radix") {
        assert_eq!(bigint::split_radix(2), (0, 1));
        assert_eq!(bigint::split_radix(4), (0, 2));
        assert_eq!(bigint::split_radix(8), (0, 3));
        assert_eq!(bigint::split_radix(16), (0, 4));
        assert_eq!(bigint::split_radix(32), (0, 5));
        assert_eq!(bigint::split_radix(14), (7, 1));
    }
}
