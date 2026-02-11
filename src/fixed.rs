use crate::base;

#[derive(Clone)]
pub struct FixedBitMap<const BLOCKS: usize> {
    blocks: [usize; BLOCKS],
}

impl<const BLOCKS: usize> FixedBitMap<BLOCKS> {
    const fn new() -> Self {
        Self {
            blocks: [0; BLOCKS],
        }
    }

    #[inline(always)]
    pub const fn set(&mut self, bit: usize) {
        base::set(&mut self.blocks, bit);
    }

    #[inline(always)]
    pub const fn clear(&mut self, bit: usize) {
        base::clear(&mut self.blocks, bit);
    }

    #[inline(always)]
    pub const fn get(&self, bit: usize) -> bool {
        base::get(&self.blocks, bit)
    }

    pub const fn count_ones(&self) -> usize {
        base::count_ones(&self.blocks)
    }

    pub const fn count_zero(&self) -> usize {
        base::count_zero(&self.blocks)
    }

    pub const fn iter_range(
        &self,
        start: usize,
        len: usize,
    ) -> impl Iterator<Item = (usize, bool)> + use<'_, BLOCKS> {
        base::iter_range(&self.blocks, start, len)
    }

    pub const fn set_range(&mut self, start: usize, len: usize) {
        base::set_range(&mut self.blocks, start, len);
    }

    pub const fn clear_range(&mut self, start: usize, len: usize) {
        base::clear_range(&mut self.blocks, start, len);
    }

    pub const fn check_range_is_set(&mut self, start: usize, len: usize) -> bool {
        base::check_range_is_set(&self.blocks, start, len)
    }

    pub const fn check_range_is_unset(&mut self, start: usize, len: usize) -> bool {
        base::check_range_is_unset(&self.blocks, start, len)
    }
}

macro_rules! generate_fixed_bit_map_struct {
    (struct $name:ident<$bits:tt>) => {
        const __BLOCKS: usize = $crate::base::blocks_number_for_bits($bits);

        pub struct $name(FixedBitMap<__BLOCKS>);

        impl $name {
            pub const fn new() -> Self {
                Self(FixedBitMap::new())
            }
        }

        impl core::ops::Deref for $name {
            type Target = FixedBitMap<__BLOCKS>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl core::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    generate_fixed_bit_map_struct!(struct TestBitmap<256>);

    fn set_bits_individually(bitmap: &mut TestBitmap, start: usize, len: usize) {
        for i in 0..len {
            bitmap.set(start + i);
        }
    }

    fn clear_bits_individually(bitmap: &mut TestBitmap, start: usize, len: usize) {
        for i in 0..len {
            bitmap.clear(start + i);
        }
    }

    fn check_bits_individually(bitmap: &TestBitmap, start: usize, len: usize) -> Vec<bool> {
        (0..len).map(|i| bitmap.get(start + i)).collect()
    }

    fn check_range_all_set(bitmap: &TestBitmap, start: usize, len: usize) -> bool {
        check_bits_individually(bitmap, start, len).iter().all(|&bit| bit)
    }

    fn check_range_all_unset(bitmap: &TestBitmap, start: usize, len: usize) -> bool {
        check_bits_individually(bitmap, start, len).iter().all(|&bit| !bit)
    }

    #[test]
    fn test_set_range_matches_individual_operations() {
        let mut bitmap_range = TestBitmap::new();
        let mut bitmap_individual = TestBitmap::new();

        let test_cases = [
            (0, 1),    // single bit at start
            (10, 5),   // small range in middle
            (0, 64),   // exactly one block
            (0, 65),   // spans two blocks
            (100, 100), // arbitrary range
            (200, 56), // range at end
            (0, 256),  // entire bitmap
            (128, 128), // second half
            (50, 150), // large range spanning multiple blocks
        ];

        for (start, len) in test_cases {
            // Reset both bitmaps
            bitmap_range = TestBitmap::new();
            bitmap_individual = TestBitmap::new();

            // Set bits using range operation
            bitmap_range.set_range(start, len);

            // Set bits individually
            set_bits_individually(&mut bitmap_individual, start, len);

            // Compare all bits in the range
            for i in start..start + len {
                assert_eq!(
                    bitmap_range.get(i),
                    bitmap_individual.get(i),
                    "Mismatch at bit {} for range ({}, {})",
                    i, start, len
                );
            }

            // Also compare bits outside the range to ensure they weren't affected
            for i in 0..256 {
                if i < start || i >= start + len {
                    assert_eq!(
                        bitmap_range.get(i),
                        bitmap_individual.get(i),
                        "Unexpected change at bit {} outside range ({}, {})",
                        i, start, len
                    );
                }
            }
        }
    }

    #[test]
    fn test_clear_range_matches_individual_operations() {
        let mut bitmap_range = TestBitmap::new();
        let mut bitmap_individual = TestBitmap::new();

        let test_cases = [
            (0, 1),    // single bit at start
            (10, 5),   // small range in middle
            (0, 64),   // exactly one block
            (0, 65),   // spans two blocks
            (100, 100), // arbitrary range
            (200, 56), // range at end
            (0, 256),  // entire bitmap
            (128, 128), // second half
            (50, 150), // large range spanning multiple blocks
        ];

        for (start, len) in test_cases {
            // First set all bits
            for i in 0..256 {
                bitmap_range.set(i);
                bitmap_individual.set(i);
            }

            // Clear bits using range operation
            bitmap_range.clear_range(start, len);

            // Clear bits individually
            clear_bits_individually(&mut bitmap_individual, start, len);

            // Compare all bits in the range
            for i in start..start + len {
                assert_eq!(
                    bitmap_range.get(i),
                    bitmap_individual.get(i),
                    "Mismatch at bit {} for range ({}, {})",
                    i, start, len
                );
            }

            // Reset for next test case
            for i in 0..256 {
                bitmap_range.set(i);
                bitmap_individual.set(i);
            }
        }
    }

    #[test]
    fn test_check_range_is_set_matches_individual_operations() {
        let mut bitmap = TestBitmap::new();
        let test_cases = [
            (0, 1),    // single bit at start
            (10, 5),   // small range in middle
            (0, 64),   // exactly one block
            (0, 65),   // spans two blocks
            (100, 100), // arbitrary range
            (200, 56), // range at end
            (0, 256),  // entire bitmap
            (128, 128), // second half
            (50, 150), // large range spanning multiple blocks
        ];

        for (start, len) in test_cases {
            // Test with unset bits
            assert_eq!(
                bitmap.check_range_is_set(start, len),
                check_range_all_set(&bitmap, start, len),
                "Range check failed for unset bits ({}, {})",
                start, len
            );

            // Set bits in the range
            set_bits_individually(&mut bitmap, start, len);

            // Test with set bits
            assert_eq!(
                bitmap.check_range_is_set(start, len),
                check_range_all_set(&bitmap, start, len),
                "Range check failed for set bits ({}, {})",
                start, len
            );

            // Clear bits for next test
            clear_bits_individually(&mut bitmap, start, len);
        }
    }

    #[test]
    fn test_check_range_is_unset_matches_individual_operations() {
        let mut bitmap = TestBitmap::new();

        let test_cases = [
            (0, 1),    // single bit at start
            (10, 5),   // small range in middle
            (0, 64),   // exactly one block
            (0, 65),   // spans two blocks
            (100, 100), // arbitrary range
            (200, 56), // range at end
            (0, 256),  // entire bitmap
            (128, 128), // second half
            (50, 150), // large range spanning multiple blocks
        ];

        for (start, len) in test_cases {
            // Test with unset bits
            assert_eq!(
                bitmap.check_range_is_unset(start, len),
                check_range_all_unset(&bitmap, start, len),
                "Range check failed for unset bits ({}, {})",
                start, len
            );

            // Set bits in the range
            set_bits_individually(&mut bitmap, start, len);

            // Test with set bits
            assert_eq!(
                bitmap.check_range_is_unset(start, len),
                check_range_all_unset(&bitmap, start, len),
                "Range check failed for set bits ({}, {})",
                start, len
            );

            // Clear bits for next test
            clear_bits_individually(&mut bitmap, start, len);
        }
    }

    #[test]
    fn test_iter_range_matches_individual_operations() {
        let mut bitmap = TestBitmap::new();

        let test_cases = [
            (0, 1),    // single bit at start
            (10, 5),   // small range in middle
            (0, 64),   // exactly one block
            (0, 65),   // spans two blocks
            (100, 100), // arbitrary range
            (200, 56), // range at end
            (0, 256),  // entire bitmap
            (128, 128), // second half
            (50, 150), // large range spanning multiple blocks
        ];

        for (start, len) in test_cases {
            // Test with unset bits
            let iter_result: Vec<(usize, bool)> = bitmap.iter_range(start, len).collect();
            let individual_result: Vec<(usize, bool)> = check_bits_individually(&bitmap, start, len)
                .into_iter()
                .enumerate()
                .map(|(i, bit)| (start + i, bit))
                .collect();

            assert_eq!(
                iter_result, individual_result,
                "Iterator mismatch for unset bits ({}, {})",
                start, len
            );

            // Set some bits in the range
            for i in (0..len).step_by(3) {
                bitmap.set(start + i);
            }

            // Test with mixed bits
            let iter_result: Vec<(usize, bool)> = bitmap.iter_range(start, len).collect();
            let individual_result: Vec<(usize, bool)> = check_bits_individually(&bitmap, start, len)
                .into_iter()
                .enumerate()
                .map(|(i, bit)| (start + i, bit))
                .collect();

            assert_eq!(
                iter_result, individual_result,
                "Iterator mismatch for mixed bits ({}, {})",
                start, len
            );

            // Clear bits for next test
            clear_bits_individually(&mut bitmap, start, len);
        }
    }

    #[test]
    fn test_edge_cases() {
        let mut bitmap = TestBitmap::new();

        // Test setting and clearing at block boundaries
        let boundary_tests = [
            (63, 1),   // end of first block
            (64, 1),   // start of second block
            (62, 4),   // spans block boundary
            (127, 1),  // end of second block
            (128, 1),  // start of third block
            (126, 4),  // spans another block boundary
        ];

        for (start, len) in boundary_tests {
            // Test set_range
            bitmap.set_range(start, len);
            for i in start..start + len {
                assert!(bitmap.get(i), "Bit {} should be set", i);
            }

            // Test clear_range
            bitmap.clear_range(start, len);
            for i in start..start + len {
                assert!(!bitmap.get(i), "Bit {} should be clear", i);
            }
        }
    }

    #[test]
    fn test_range_operations_with_partial_patterns() {
        let mut bitmap = TestBitmap::new();

        // Create a pattern: set every other bit
        for i in 0..256 {
            if i % 2 == 0 {
                bitmap.set(i);
            }
        }

        let test_cases = [
            (0, 10),
            (5, 20),
            (50, 50),
            (100, 100),
            (200, 56),
        ];

        for (start, len) in test_cases {
            // Test check_range_is_set (should be false for our pattern)
            assert!(!bitmap.check_range_is_set(start, len));

            // Test check_range_is_unset (should be false for our pattern)
            assert!(!bitmap.check_range_is_unset(start, len));

            // Test iter_range matches individual checks
            let iter_result: Vec<bool> = bitmap.iter_range(start, len)
                .map(|(_, bit)| bit)
                .collect();
            let individual_result = check_bits_individually(&bitmap, start, len);

            assert_eq!(iter_result, individual_result);
        }
    }

    #[test]
    fn test_count_operations_after_range_operations() {
        let mut bitmap = TestBitmap::new();

        let test_cases = [
            (0, 64),
            (64, 64),
            (128, 64),
            (50, 100),
        ];

        for (start, len) in test_cases {
            let initial_count = bitmap.count_ones();

            // Set range and verify count
            bitmap.set_range(start, len);
            assert_eq!(bitmap.count_ones(), initial_count + len);

            // Clear range and verify count
            bitmap.clear_range(start, len);
            assert_eq!(bitmap.count_ones(), initial_count);
        }
    }
}
