use std::fmt;
use std::iter::FusedIterator;
use std::ops::{BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr, ShrAssign};

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bitmap256([u64; 4]);

impl Bitmap256 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_value<T: Into<u128>>(value: T) -> Self {
        let value: u128 = value.into();
        Bitmap256([0u64, 0u64, (value >> 64) as u64, (value as u64)])
    }

    pub fn from_bit(value: u8) -> Self {
        Bitmap256([0u64, 0u64, 0u64, 1u64]) << value
    }

    pub fn clear_bit(&mut self, bit: u8) {
        self.0[3 - (bit >> 6) as usize] &= !(1u64 << (bit % 64));
    }

    pub fn set_bit(&mut self, bit: u8) {
        self.0[3 - (bit >> 6) as usize] |= 1u64 << (bit % 64);
    }

    pub fn flip_bit(&mut self, bit: u8) {
        self.0[3 - (bit >> 6) as usize] ^= 1u64 << (bit % 64);
    }

    pub fn leading_zeros(&self) -> u16 {
        let mut result: u16 = 0;
        for word in 0..4 {
            if self.0[word] == 0 {
                result += 64;
            } else {
                result += self.0[word].leading_zeros() as u16;
                break;
            }
        }
        result
    }

    pub fn trailing_zeros(&self) -> u16 {
        let mut result: u16 = 0;
        for word in (0..4).rev() {
            if self.0[word] == 0 {
                result += 64;
            } else {
                result += self.0[word].trailing_zeros() as u16;
                break;
            }
        }
        result
    }

    pub fn most_significant_bit(&self) -> Option<u8> {
        let zeros = self.leading_zeros();
        debug_assert!(
            zeros <= 256,
            "leading_zeros() returned {zeros}, but a 256-bit bitmap has at most 256"
        );
        match zeros {
            256.. => None,
            val => Some(255 - val as u8),
        }
    }

    pub fn least_significant_bit(&self) -> Option<u8> {
        let zeros = self.trailing_zeros();
        debug_assert!(
            zeros <= 256,
            "trailing_zeros() returned {zeros}, but a 256-bit bitmap has at most 256"
        );
        match zeros {
            256.. => None,
            val => Some(val as u8),
        }
    }

    pub fn count_ones(&self) -> usize {
        let mut ones: usize = 0;
        for iter in 0..4 {
            ones += self.0[iter].count_ones() as usize;
        }
        ones
    }

    pub fn ones(&self) -> Ones {
        Ones(*self)
    }
}

impl fmt::Binary for Bitmap256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Word 0 is the most significant, so printing words in array
        // order gives the conventional MSB-on-the-left layout.
        for word in self.0 {
            write!(f, "{:064b}", word)?;
        }
        Ok(())
    }
}

impl fmt::Display for Bitmap256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.ones()).finish()
    }
}

impl Not for Bitmap256 {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for iter in 0..4 {
            self.0[iter] = !self.0[iter];
        }
        self
    }
}

impl BitXor for Bitmap256 {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        for iter in 0..4 {
            self.0[iter] ^= rhs.0[iter];
        }
        self
    }
}

impl BitXorAssign for Bitmap256 {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl Shl<u8> for Bitmap256 {
    type Output = Self;
    fn shl(mut self, rhs: u8) -> Self::Output {
        let bit_offset = rhs % 64;
        let word_offset = (rhs as isize) >> 6; // Divide by 64
        for iter in 0..4 {
            // Apply chunk offset first
            let source_word_index = iter + word_offset;
            if let 0..4 = source_word_index {
                self.0[iter as usize] = self.0[source_word_index as usize];
            } else {
                self.0[iter as usize] = 0
            }
            // Now apply the additional offset within the chunk itself
            let spill_index = source_word_index + 1;
            if bit_offset > 0 {
                self.0[iter as usize] <<= bit_offset;
                if let 0..4 = spill_index {
                    let spill_over = self.0[spill_index as usize] >> (64 - bit_offset);
                    self.0[iter as usize] |= spill_over;
                }
            }
        }
        self
    }
}

impl ShlAssign<u8> for Bitmap256 {
    fn shl_assign(&mut self, rhs: u8) {
        *self = *self << rhs;
    }
}

impl Shr<u8> for Bitmap256 {
    type Output = Self;
    fn shr(mut self, rhs: u8) -> Self::Output {
        let bit_offset = rhs % 64;
        let word_offset = (rhs as isize) >> 6; // Divide by 64
        for iter in (0..4).rev() {
            // Apply chunk offset first
            let source_word_index = iter - word_offset;
            if let 0..4 = source_word_index {
                self.0[iter as usize] = self.0[source_word_index as usize];
            } else {
                self.0[iter as usize] = 0
            }
            // Now apply the additional offset within the chunk itself
            let spill_index = source_word_index - 1;
            if bit_offset > 0 {
                self.0[iter as usize] >>= bit_offset;
                if let 0..4 = spill_index {
                    let spill_over = self.0[spill_index as usize] << (64 - bit_offset);
                    self.0[iter as usize] |= spill_over;
                }
            }
        }
        self
    }
}

impl ShrAssign<u8> for Bitmap256 {
    fn shr_assign(&mut self, rhs: u8) {
        *self = *self >> rhs;
    }
}

impl IntoIterator for Bitmap256 {
    type Item = u8;

    type IntoIter = Ones;

    fn into_iter(self) -> Self::IntoIter {
        self.ones()
    }
}

impl IntoIterator for &Bitmap256 {
    type Item = u8;

    type IntoIter = Ones;

    fn into_iter(self) -> Self::IntoIter {
        self.ones()
    }
}

#[derive(Clone, Debug)]
pub struct Ones(Bitmap256);

impl Iterator for Ones {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let least_significant = self.0.least_significant_bit();
        if let Some(x) = least_significant {
            self.0 ^= Bitmap256::from_bit(x);
        }
        least_significant
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.0.count_ones();
        (size, Some(size))
    }
}

impl DoubleEndedIterator for Ones {
    fn next_back(&mut self) -> Option<Self::Item> {
        let most_significant = self.0.most_significant_bit();
        if let Some(x) = most_significant {
            self.0 ^= Bitmap256::from_bit(x);
        }
        most_significant
    }
}

impl FusedIterator for Ones {}

impl ExactSizeIterator for Ones {}

#[cfg(test)]
mod tests {
    use super::*;

    // Constructors

    #[test]
    fn new_equals_default() {
        let from_new = Bitmap256::new();
        let from_default = Bitmap256::default();
        let manual = Bitmap256([0u64, 0u64, 0u64, 0u64]);
        assert_eq!(from_new, from_default);
        assert_eq!(from_new, manual);
    }

    #[test]
    fn from_value_0_eq_new() {
        let new = Bitmap256::new();
        let from_zero = Bitmap256::from_value(0u32);
        assert_eq!(new, from_zero);
    }

    #[test]
    fn from_max_u64_fills_only_word_3() {
        let from_max64 = Bitmap256::from_value(u64::MAX);
        assert_eq!(
            from_max64.0[0], 0,
            "Word 0 value is {}, expected 0",
            from_max64.0[0]
        );
        assert_eq!(
            from_max64.0[1], 0,
            "Word 1 value is {}, expected 0",
            from_max64.0[1]
        );
        assert_eq!(
            from_max64.0[2], 0,
            "Word 2 value is {}, expected 0",
            from_max64.0[2]
        );
        assert!(
            from_max64.0[3] > 0,
            "Word 3 value is {}, expected gt 0",
            from_max64.0[3]
        );
    }

    #[test]
    fn from_max_u128_fills_only_words_2_3() {
        let from_max128 = Bitmap256::from_value(u128::MAX);
        assert_eq!(
            from_max128.0[0], 0,
            "Word 0 value is {}, expected 0",
            from_max128.0[0]
        );
        assert_eq!(
            from_max128.0[1], 0,
            "Word 1 value is {}, expected 0",
            from_max128.0[1]
        );
        assert!(
            from_max128.0[2] > 0,
            "Word 2 value is {}, expected gt 0",
            from_max128.0[2]
        );
        assert!(
            from_max128.0[3] > 0,
            "Word 3 value is {}, expected gt 0",
            from_max128.0[3]
        );
    }

    #[test]
    fn from_value_same_result_with_any_unsigned_int_type() {
        let test_value: u8 = 200;
        let with_u8 = Bitmap256::from_value(test_value);
        let with_u16 = Bitmap256::from_value(test_value as u16);
        let with_u32 = Bitmap256::from_value(test_value as u32);
        let with_u64 = Bitmap256::from_value(test_value as u64);
        let with_u128 = Bitmap256::from_value(test_value as u128);
        assert_eq!(with_u8, with_u16);
        assert_eq!(with_u16, with_u32);
        assert_eq!(with_u32, with_u64);
        assert_eq!(with_u64, with_u128);
    }

    #[test]
    fn from_bit_constructs_correctly() {
        let from_0 = Bitmap256::from_bit(0);
        let from_63 = Bitmap256::from_bit(63);
        let from_64 = Bitmap256::from_bit(64);
        let from_255 = Bitmap256::from_bit(255);
        assert_eq!(
            from_0,
            Bitmap256([0u64, 0u64, 0u64, 1u64]),
            "from_bit(0) returns an incorrect value: {:b}",
            from_0
        );
        assert_eq!(
            from_63,
            Bitmap256([0u64, 0u64, 0u64, 1u64 << 63]),
            "from_bit(63) returns an incorrect value: {:b}",
            from_63
        );
        assert_eq!(
            from_64,
            Bitmap256([0u64, 0u64, 1u64, 0u64]),
            "from_bit(64) returns an incorrect value: {:b}",
            from_64
        );
        assert_eq!(
            from_255,
            Bitmap256([1u64 << 63, 0u64, 0u64, 0u64]),
            "from_bit(255) returns an incorrect value: {:b}",
            from_255
        );
    }

    #[test]
    fn from_bit_equals_shl() {
        for i in 0..127 {
            let bitmap_from_bit = Bitmap256::from_bit(i);
            let bitmap_from_shl = Bitmap256([0u64, 0u64, 0u64, 1u64]) << i;
            assert_eq!(bitmap_from_bit, bitmap_from_shl);
        }
    }

    // Not

    #[test]
    fn not_flips_the_whole_bitmap() {
        let negated = !Bitmap256::new();
        assert_eq!(negated, Bitmap256([u64::MAX, u64::MAX, u64::MAX, u64::MAX]));
    }

    #[test]
    fn double_negation_returns_the_original() {
        let original = Bitmap256::from_value(296483u128);
        let negated = !original;
        assert_eq!(original, !negated);
    }

    // Single bit operations

    #[test]
    fn clear_single_bit() {
        for bit in 0..=255 {
            let mut bitmap = Bitmap256::from_bit(bit);
            bitmap.clear_bit(bit);
            assert_eq!(bitmap, Bitmap256([0u64, 0u64, 0u64, 0u64]));
        }
    }

    #[test]
    fn set_single_bit() {
        for bit in 0..=255 {
            let mut bitmap = Bitmap256([0u64, 0u64, 0u64, 0u64]);
            bitmap.set_bit(bit);
            assert_eq!(bitmap, Bitmap256::from_bit(bit));
        }
    }

    #[test]
    fn flip_single_bit() {
        for bit in 0..=255 {
            let mut bitmap = Bitmap256::from_bit(bit);
            bitmap.flip_bit(bit);
            assert_eq!(bitmap, Bitmap256([0u64, 0u64, 0u64, 0u64]));
        }
    }

    #[test]
    fn flip_single_bit_twice_returns_original() {
        for bit in 0..=255 {
            let bitmap = Bitmap256::from_bit(bit);
            let mut flipped = bitmap;
            flipped.flip_bit(bit);
            flipped.flip_bit(bit);
            assert_eq!(flipped, bitmap);
        }
    }

    // Xor

    #[test]
    fn xor_with_itself_returns_empty_bitmap() {
        let original = Bitmap256::from_value(1329923u128);
        let xor_with_itself = original ^ original;
        assert_eq!(xor_with_itself, Bitmap256([0u64, 0u64, 0u64, 0u64]));
    }

    #[test]
    fn xor_with_empty_bitmap_returns_the_original() {
        let original = Bitmap256::from_value(9923u128);
        let empty = Bitmap256::new();
        let xor_with_empty = original ^ empty;
        assert_eq!(xor_with_empty, original);
    }

    #[test]
    fn xor_two_single_bit_maps_returns_has_those_bits_set() {
        let first = Bitmap256::from_bit(10);
        let second = Bitmap256::from_bit(200);
        let xor = first ^ second;
        assert_eq!(xor, Bitmap256([256u64, 0u64, 0u64, 1024u64]));
    }

    // Shl

    #[test]
    fn shl_by_zero_has_not_effect() {
        let bitmap = Bitmap256::from_value(109283713013710371203917937190371765768u128);
        let shifted_by_zero = bitmap << 0;
        assert_eq!(bitmap, shifted_by_zero);
    }

    #[test]
    fn shl_carry_across_word_boundary() {
        let bitmap = Bitmap256::from_bit(63);
        assert_eq!(bitmap << 1, Bitmap256::from_bit(64));
    }

    #[test]
    fn from_bit_255_left_shifted_is_empty() {
        let bitmap = Bitmap256::from_bit(255);
        let shifted = bitmap << 1;
        assert_eq!(shifted, Bitmap256([0u64, 0u64, 0u64, 0u64]));
    }

    #[test]
    fn shl_by_64_128_192_moves_whole_words() {
        let word_a = 237468u64;
        let word_b = 120u64;
        let word_c = 0u64;
        let word_d = u64::MAX;
        let bitmap = Bitmap256([word_a, word_b, word_c, word_d]);
        assert_eq!(bitmap << 64, Bitmap256([word_b, word_c, word_d, 0u64]));
        assert_eq!(bitmap << 128, Bitmap256([word_c, word_d, 0u64, 0u64]));
        assert_eq!(bitmap << 192, Bitmap256([word_d, 0u64, 0u64, 0u64]));
    }

    #[test]
    fn shl_by_65_combines_word_move_with_carry() {
        let word_a = 0u64;
        let word_b = 120u64;
        let word_c = 237468u64;
        let word_d = u64::MAX;
        let bitmap = Bitmap256([word_a, word_b, word_c, word_d]);
        let shifted = bitmap << 65;
        let shifted_manually = Bitmap256([
            (word_b << 1) | word_c >> 63,
            (word_c << 1) | word_d >> 63,
            word_d << 1,
            0u64,
        ]);
        assert_eq!(shifted, shifted_manually);
    }

    // Shr

    #[test]
    fn shr_by_zero_has_not_effect() {
        let bitmap = Bitmap256::from_value(109283713013710371203917937190371765768u128);
        let shifted_by_zero = bitmap >> 0;
        assert_eq!(bitmap, shifted_by_zero);
    }

    #[test]
    fn shr_carry_across_word_boundary() {
        let bitmap = Bitmap256::from_bit(64);
        assert_eq!(bitmap >> 1, Bitmap256::from_bit(63));
    }

    #[test]
    fn from_bit_0_right_shifted_is_empty() {
        let bitmap = Bitmap256::from_bit(0);
        let shifted = bitmap >> 1;
        assert_eq!(shifted, Bitmap256([0u64, 0u64, 0u64, 0u64]));
    }

    #[test]
    fn shr_by_64_128_192_moves_whole_words() {
        let word_a = 237468u64;
        let word_b = 120u64;
        let word_c = 0u64;
        let word_d = u64::MAX;
        let bitmap = Bitmap256([word_a, word_b, word_c, word_d]);
        assert_eq!(bitmap >> 64, Bitmap256([0u64, word_a, word_b, word_c]));
        assert_eq!(bitmap >> 128, Bitmap256([0u64, 0u64, word_a, word_b]));
        assert_eq!(bitmap >> 192, Bitmap256([0u64, 0u64, 0u64, word_a]));
    }

    #[test]
    fn shr_by_65_combines_word_move_with_carry() {
        let word_a = 0u64;
        let word_b = 120u64;
        let word_c = 237468u64;
        let word_d = u64::MAX;
        let bitmap = Bitmap256([word_a, word_b, word_c, word_d]);
        let shifted = bitmap >> 65;
        let shifted_manually = Bitmap256([
            0u64,
            word_a >> 1,
            (word_b >> 1) | (word_a >> 63) << 63,
            (word_c >> 1) | (word_b >> 63) << 63,
        ]);
        assert_eq!(shifted, shifted_manually);
    }

    // Shift relationships

    #[test]
    fn shl_and_shr_by_the_same_restores_original_positions() {
        let bitmap = Bitmap256::from_bit(25);
        assert_eq!((bitmap << 5) >> 5, bitmap);
    }

    #[test]
    fn shifting_by_a_then_b_equals_shifting_by_a_plus_b() {
        let bitmap = Bitmap256::from_bit(25);
        let a: u8 = 5;
        let b: u8 = 6;
        assert_eq!((bitmap << a) << b, bitmap << (a + b));
    }

    // Zero counting

    #[test]
    fn leading_zeros_returns_256_on_empty_bitmap() {
        let empty = Bitmap256::new();
        assert_eq!(empty.leading_zeros(), 256);
    }

    #[test]
    fn leading_zeros_returns_0_on_full_bitmap() {
        let full = !Bitmap256::new();
        assert_eq!(full.leading_zeros(), 0);
    }

    #[test]
    fn leading_zeros_is_255_minus_n_on_from_bits_n() {
        for n in 0u16..255u16 {
            let bitmap = Bitmap256::from_bit(n as u8);
            assert_eq!(bitmap.leading_zeros(), 255u16 - n);
        }
    }

    #[test]
    fn trailing_zeros_returns_256_on_empty_bitmap() {
        let empty = Bitmap256::new();
        assert_eq!(empty.trailing_zeros(), 256);
    }

    #[test]
    fn trailing_zeros_returns_0_on_full_bitmap() {
        let full = !Bitmap256::new();
        assert_eq!(full.trailing_zeros(), 0);
    }

    #[test]
    fn trailing_zeros_is_n_on_from_bits_n() {
        for n in 0u16..255u16 {
            let bitmap = Bitmap256::from_bit(n as u8);
            assert_eq!(bitmap.trailing_zeros(), n);
        }
    }

    #[test]
    fn most_significant_bit_returns_none_on_empty() {
        let empty = Bitmap256::new();
        assert_eq!(empty.most_significant_bit(), None);
    }

    #[test]
    fn most_significant_bit_returns_some_on_non_empty() {
        for n in 0..255 {
            let bitmap = Bitmap256::from_bit(n);
            assert_eq!(bitmap.most_significant_bit(), Some(n));
        }
    }

    #[test]
    fn most_significant_bit_returns_the_highest_on_multi_bit_maps() {
        let bitmap_5 = Bitmap256::from_bit(5);
        let bitmap_200 = Bitmap256::from_bit(200);
        let with_2_set_bits = bitmap_5 ^ bitmap_200;
        assert_eq!(with_2_set_bits.most_significant_bit().unwrap(), 200u8);
    }

    #[test]
    fn least_significant_bit_returns_none_on_empty() {
        let empty = Bitmap256::new();
        assert_eq!(empty.least_significant_bit(), None);
    }

    #[test]
    fn least_significant_bit_returns_some_on_non_empty() {
        for n in 0..255 {
            let bitmap = Bitmap256::from_bit(n);
            assert_eq!(bitmap.least_significant_bit(), Some(n));
        }
    }

    #[test]
    fn least_significant_bit_returns_the_lowest_on_multi_bit_maps() {
        let bitmap_5 = Bitmap256::from_bit(5);
        let bitmap_200 = Bitmap256::from_bit(200);
        let with_2_set_bits = bitmap_5 ^ bitmap_200;
        assert_eq!(with_2_set_bits.least_significant_bit().unwrap(), 5u8);
    }

    // Count ones

    #[test]
    fn count_ones_returns_zero_on_empty() {
        let empty = Bitmap256::new();
        assert_eq!(empty.count_ones(), 0);
    }

    #[test]
    fn count_ones_returns_256_on_full() {
        let full = !Bitmap256::new();
        assert_eq!(full.count_ones(), 256);
    }

    #[test]
    fn count_ones_returns_1_for_every_from_bit_n() {
        for n in 0..255 {
            assert_eq!(Bitmap256::from_bit(n).count_ones(), 1);
        }
    }

    #[test]
    fn count_ones_matches_the_primitive_count_ones() {
        let test_values: [u128; 11] = [
            0,
            (u8::MAX / 2) as u128,
            u8::MAX as u128,
            (u16::MAX / 2) as u128,
            u16::MAX as u128,
            (u32::MAX / 2) as u128,
            u32::MAX as u128,
            (u64::MAX / 2) as u128,
            u64::MAX as u128,
            u128::MAX / 2,
            u128::MAX,
        ];
        for n in test_values {
            let primitive = n.count_ones();
            let bitmap255 = Bitmap256::from_value(n).count_ones();
            assert_eq!(bitmap255, primitive as usize);
        }
    }
}
