//! Flexible bitmap with dynamic resizing
//!
//! This module provides `FlexBitMap`, a bitmap that can grow dynamically
//! as needed. Unlike `FixedBitMap`, it doesn't have a fixed capacity and
//! will automatically expand when setting bits beyond the current capacity.
//!
//! # Examples
//!
//! ```rust
//! use ranged_bitmap::FlexBitMap;
//!
//! let mut bitmap = FlexBitMap::new();
//! bitmap.set(1000); // Automatically grows to accommodate bit 1000
//! assert!(bitmap.get(1000));
//! ```

extern crate alloc;

use crate::base::{
    bitmap_check_range_is_set, bitmap_check_range_is_unset, bitmap_clear, bitmap_clear_range,
    bitmap_count_ones, bitmap_count_zeros, bitmap_get, bitmap_set, bitmap_set_range,
    calc_block_and_slot, BITS_IN_USIZE,
};
use alloc::boxed::Box;
use alloc::vec;

/// A flexible bitmap that can grow dynamically.
///
/// This bitmap uses a boxed slice internally and automatically grows when needed.
/// It's designed for situations where the maximum size isn't known at compile time.
///
/// # Performance
///
/// - Growing incurs allocation cost, but subsequent operations are O(1)
/// - Uses the same high-performance range operations as `FixedBitMap`
/// - Memory usage grows in 64-bit blocks
///
/// # Examples
///
/// ```rust
/// use ranged_bitmap::FlexBitMap;
///
/// let mut bitmap = FlexBitMap::new();
///
/// // Set individual bits
/// bitmap.set(10);
/// bitmap.set(20);
///
/// // Set ranges - automatically grows if needed
/// bitmap.set_range(100, 50);
///
/// // Check bits
/// assert!(bitmap.get(10));
/// assert!(bitmap.get(125));
/// assert!(!bitmap.get(200));
///
/// // Count bits
/// assert_eq!(bitmap.count_ones(), 52);
/// ```
pub struct FlexBitMap {
    data: Box<[usize]>,
}

impl FlexBitMap {
    /// Creates a new empty bitmap.
    ///
    /// The bitmap starts empty and will grow as needed when bits are set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ranged_bitmap::FlexBitMap;
    ///
    /// let bitmap = FlexBitMap::new();
    /// assert_eq!(bitmap.count_ones(), 0);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self { data: Box::new([]) }
    }

    /// Ensures the bitmap has enough capacity for the specified bit.
    ///
    /// This method grows the internal slice if necessary to accommodate
    /// the specified bit position.
    fn ensure_capacity(&mut self, bit: usize) {
        let (block, _) = calc_block_and_slot(bit);
        if block >= self.data.len() {
            let mut new_data = vec![0; block + 1];

            new_data[..self.data.len()].copy_from_slice(&self.data);

            self.data = new_data.into_boxed_slice();
        }
    }

    /// Ensures the bitmap has enough capacity for a range ending at `start + len`.
    ///
    /// This method grows the internal slice if necessary to accommodate
    /// the specified range.
    fn ensure_range_capacity(&mut self, start: usize, len: usize) {
        if len == 0 {
            return;
        }

        let end_bit = start + len - 1;
        let (block, _) = calc_block_and_slot(end_bit);

        if block >= self.data.len() {
            let mut new_data = vec![0; block + 1];

            new_data[..self.data.len()].copy_from_slice(&self.data);

            self.data = new_data.into_boxed_slice();
        }
    }

    /// Sets a bit to 1.
    ///
    /// Automatically grows the bitmap if the bit is beyond current capacity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ranged_bitmap::FlexBitMap;
    ///
    /// let mut bitmap = FlexBitMap::new();
    ///
    /// bitmap.set(42);
    ///
    /// assert!(bitmap.get(42));
    /// ```
    #[inline]
    pub fn set(&mut self, bit: usize) {
        self.ensure_capacity(bit);

        bitmap_set(&mut self.data, bit);
    }

    /// Clears a bit to 0.
    ///
    /// If the bit is beyond current capacity, this operation does nothing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ranged_bitmap::FlexBitMap;
    ///
    /// let mut bitmap = FlexBitMap::new();
    ///
    /// bitmap.set(42);
    /// bitmap.clear(42);
    ///
    /// assert!(!bitmap.get(42));
    /// ```
    #[inline]
    pub fn clear(&mut self, bit: usize) {
        let (block, _) = calc_block_and_slot(bit);
        if block < self.data.len() {
            bitmap_clear(&mut self.data, bit);
        }
    }

    /// Gets the value of a bit.
    ///
    /// Returns `false` for bits beyond current capacity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ranged_bitmap::FlexBitMap;
    ///
    /// let mut bitmap = FlexBitMap::new();
    ///
    /// assert!(!bitmap.get(100)); // Beyond capacity, returns false
    ///
    /// bitmap.set(100);
    ///
    /// assert!(bitmap.get(100));
    /// ```
    #[inline]
    pub fn get(&self, bit: usize) -> bool {
        let (block, _) = calc_block_and_slot(bit);
        if block < self.data.len() {
            bitmap_get(&self.data, bit)
        } else {
            false
        }
    }

    /// Counts the number of set bits in the bitmap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ranged_bitmap::FlexBitMap;
    ///
    /// let mut bitmap = FlexBitMap::new();
    ///
    /// bitmap.set(10);
    /// bitmap.set(20);
    ///
    /// assert_eq!(bitmap.count_ones(), 2);
    /// ```
    #[inline]
    pub fn count_ones(&self) -> usize {
        bitmap_count_ones(&self.data)
    }

    /// Counts the number of clear bits in the bitmap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ranged_bitmap::FlexBitMap;
    ///
    /// let mut bitmap = FlexBitMap::new();
    ///
    /// bitmap.set_range(0, 10);
    ///
    /// // count_zeros counts all zeros in allocated blocks, not just the set range
    /// assert_eq!(bitmap.count_zeros(), bitmap.capacity() - 10);
    /// ```
    #[inline]
    pub fn count_zeros(&self) -> usize {
        bitmap_count_zeros(&self.data)
    }

    /// Iterates over a range of bits, returning (position, value) pairs.
    ///
    /// For bits beyond current capacity, the value is always `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ranged_bitmap::FlexBitMap;
    ///
    /// let mut bitmap = FlexBitMap::new();
    ///
    /// bitmap.set(5);
    /// bitmap.set(7);
    ///
    /// let bits: Vec<_> = bitmap.iter_range(4, 4).collect();
    ///
    /// assert_eq!(bits, vec![(4, false), (5, true), (6, false), (7, true)]);
    /// ```
    pub fn iter_range(&self, start: usize, len: usize) -> impl Iterator<Item = (usize, bool)> + '_ {
        FlexRangeIter {
            bitmap: self,
            start,
            len,
            current: 0,
        }
    }
}

/// Iterator for `FlexBitMap` range operations
struct FlexRangeIter<'a> {
    bitmap: &'a FlexBitMap,
    start: usize,
    len: usize,
    current: usize,
}

impl Iterator for FlexRangeIter<'_> {
    type Item = (usize, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.len {
            return None;
        }

        let pos = self.start + self.current;
        let value = self.bitmap.get(pos);

        self.current += 1;

        Some((pos, value))
    }
}

impl FlexBitMap {
    /// Sets all bits in a range to 1.
    ///
    /// Automatically grows the bitmap if the range extends beyond current capacity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ranged_bitmap::FlexBitMap;
    ///
    /// let mut bitmap = FlexBitMap::new();
    /// bitmap.set_range(100, 50); // Automatically grows
    ///
    /// for i in 100..150 {
    ///     assert!(bitmap.get(i));
    /// }
    /// ```
    #[inline]
    pub fn set_range(&mut self, start: usize, len: usize) {
        if len == 0 {
            return;
        }

        self.ensure_range_capacity(start, len);

        bitmap_set_range(&mut self.data, start, len);
    }

    /// Clears all bits in a range to 0.
    ///
    /// For bits beyond current capacity, this operation does nothing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ranged_bitmap::FlexBitMap;
    ///
    /// let mut bitmap = FlexBitMap::new();
    ///
    /// bitmap.set_range(0, 100);
    /// bitmap.clear_range(50, 25);
    ///
    /// // Bits 50-74 should be clear, others set
    /// assert!(!bitmap.get(60));
    /// assert!(bitmap.get(40));
    /// assert!(bitmap.get(80));
    /// ```
    #[inline]
    pub fn clear_range(&mut self, start: usize, len: usize) {
        if len == 0 {
            return;
        }

        let capacity = self.capacity();

        // If range starts beyond capacity, do nothing
        if start >= capacity {
            return;
        }

        // If range extends beyond capacity, only clear within capacity
        let actual_len = if start + len > capacity {
            capacity - start
        } else {
            len
        };

        bitmap_clear_range(&mut self.data, start, actual_len);
    }

    /// Checks if all bits in a range are set to 1.
    ///
    /// For bits beyond current capacity, they are considered unset.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ranged_bitmap::FlexBitMap;
    ///
    /// let mut bitmap = FlexBitMap::new();
    ///
    /// bitmap.set_range(10, 5);
    ///
    /// assert!(bitmap.check_range_is_set(10, 5));
    /// assert!(!bitmap.check_range_is_set(10, 6)); // Bit 15 is not set
    /// ```
    #[inline]
    pub fn check_range_is_set(&self, start: usize, len: usize) -> bool {
        if len == 0 {
            return true;
        }

        let end_bit = start + len;
        let capacity = self.capacity();
        if end_bit > capacity {
            return false;
        }

        bitmap_check_range_is_set(&self.data, start, len)
    }

    /// Checks if all bits in a range are clear to 0.
    ///
    /// For bits beyond current capacity, they are considered clear.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ranged_bitmap::FlexBitMap;
    ///
    /// let mut bitmap = FlexBitMap::new();
    ///
    /// bitmap.set_range(10, 5);
    ///
    /// assert!(bitmap.check_range_is_unset(0, 10));
    /// assert!(!bitmap.check_range_is_unset(10, 5));
    /// ```
    #[inline]
    pub fn check_range_is_unset(&self, start: usize, len: usize) -> bool {
        if len == 0 {
            return true;
        }

        let end_bit = start + len;
        let capacity = self.capacity();

        let actual_len = if end_bit > capacity {
            if start >= capacity {
                return true; // Range starts beyond capacity, all bits are considered clear
            }

            capacity - start
        } else {
            len
        };

        bitmap_check_range_is_unset(&self.data, start, actual_len)
    }

    /// Returns the current capacity in bits.
    ///
    /// This is the highest bit position that can be accessed without growing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ranged_bitmap::FlexBitMap;
    ///
    /// let mut bitmap = FlexBitMap::new();
    /// assert_eq!(bitmap.capacity(), 0);
    ///
    /// bitmap.set(100);
    /// assert!(bitmap.capacity() >= 100);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.len() * BITS_IN_USIZE
    }

    /// Returns the number of allocated blocks.
    ///
    /// Each block is `usize` bits (typically 64 bits).
    #[inline]
    pub fn blocks(&self) -> usize {
        self.data.len()
    }
}

impl Default for FlexBitMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    fn set_bits_individually(bitmap: &mut super::FlexBitMap, start: usize, len: usize) {
        for i in 0..len {
            bitmap.set(start + i);
        }
    }

    fn clear_bits_individually(bitmap: &mut super::FlexBitMap, start: usize, len: usize) {
        for i in 0..len {
            bitmap.clear(start + i);
        }
    }

    fn check_bits_individually(bitmap: &super::FlexBitMap, start: usize, len: usize) -> Vec<bool> {
        (0..len).map(|i| bitmap.get(start + i)).collect()
    }

    fn check_range_all_set(bitmap: &super::FlexBitMap, start: usize, len: usize) -> bool {
        check_bits_individually(bitmap, start, len)
            .iter()
            .all(|&bit| bit)
    }

    fn check_range_all_unset(bitmap: &super::FlexBitMap, start: usize, len: usize) -> bool {
        check_bits_individually(bitmap, start, len)
            .iter()
            .all(|&bit| !bit)
    }

    #[test]
    fn test_growth_on_set() {
        let mut bitmap = super::FlexBitMap::new();
        assert_eq!(bitmap.capacity(), 0);

        // Setting a bit should grow the bitmap
        bitmap.set(10);
        assert!(bitmap.capacity() >= 11);
        assert!(bitmap.get(10));

        // Setting a much higher bit should grow significantly
        bitmap.set(1000);
        assert!(bitmap.capacity() >= 1001);
        assert!(bitmap.get(1000));
        assert!(bitmap.get(10)); // Previous bit should still be set
    }

    #[test]
    fn test_growth_on_set_range() {
        let mut bitmap = super::FlexBitMap::new();
        assert_eq!(bitmap.capacity(), 0);

        // Setting a range should grow the bitmap
        bitmap.set_range(100, 50);
        assert!(bitmap.capacity() >= 150);

        // All bits in range should be set
        for i in 100..150 {
            assert!(bitmap.get(i));
        }
    }

    #[test]
    fn test_set_range_matches_individual_operations() {
        let test_cases = [
            (0, 1),     // single bit at start
            (10, 5),    // small range in middle
            (0, 64),    // exactly one block
            (0, 65),    // spans two blocks
            (100, 100), // arbitrary range
            (200, 56),  // range at end
            (0, 256),   // entire bitmap
            (128, 128), // second half
            (50, 150),  // large range spanning multiple blocks
        ];

        for (start, len) in test_cases {
            // Clear both bitmaps
            let mut bitmap_range = super::FlexBitMap::new();
            let mut bitmap_individual = super::FlexBitMap::new();

            // Set range
            bitmap_range.set_range(start, len);

            // Set bits individually
            set_bits_individually(&mut bitmap_individual, start, len);

            // Compare all bits in the range
            for i in 0..(start + len + 10) {
                assert_eq!(
                    bitmap_range.get(i),
                    bitmap_individual.get(i),
                    "Mismatch at bit {i} for range ({start}, {len})"
                );
            }
        }
    }

    #[test]
    fn test_clear_range_matches_individual_operations() {
        let mut bitmap_range = super::FlexBitMap::new();
        let mut bitmap_individual = super::FlexBitMap::new();

        let test_cases = [
            (0, 1),     // single bit at start
            (10, 5),    // small range in middle
            (0, 64),    // exactly one block
            (0, 65),    // spans two blocks
            (100, 100), // arbitrary range
            (200, 56),  // range at end
            (0, 256),   // entire bitmap
            (128, 128), // second half
            (50, 150),  // large range spanning multiple blocks
        ];

        for (start, len) in test_cases {
            // Set all bits first
            bitmap_range.set_range(0, start + len + 10);
            bitmap_individual.set_range(0, start + len + 10);

            // Clear range
            bitmap_range.clear_range(start, len);

            // Clear bits individually
            clear_bits_individually(&mut bitmap_individual, start, len);

            // Compare all bits in the range
            for i in 0..(start + len + 10) {
                assert_eq!(
                    bitmap_range.get(i),
                    bitmap_individual.get(i),
                    "Mismatch at bit {i} for range ({start}, {len})"
                );
            }
        }
    }

    #[test]
    fn test_check_range_is_set_matches_individual_operations() {
        let mut bitmap = super::FlexBitMap::new();
        let test_cases = [
            (0, 1),     // single bit at start
            (10, 5),    // small range in middle
            (0, 64),    // exactly one block
            (0, 65),    // spans two blocks
            (100, 100), // arbitrary range
            (200, 56),  // range at end
            (0, 256),   // entire bitmap
            (128, 128), // second half
            (50, 150),  // large range spanning multiple blocks
        ];

        for (start, len) in test_cases {
            // Test with unset bits
            assert_eq!(
                bitmap.check_range_is_set(start, len),
                check_range_all_set(&bitmap, start, len),
                "Range check failed for unset bits ({start}, {len})"
            );

            // Set bits in the range
            set_bits_individually(&mut bitmap, start, len);

            // Test with set bits
            assert_eq!(
                bitmap.check_range_is_set(start, len),
                check_range_all_set(&bitmap, start, len),
                "Range check failed for set bits ({start}, {len})"
            );

            // Clear bits for next test
            clear_bits_individually(&mut bitmap, start, len);
        }
    }

    #[test]
    fn test_check_range_is_unset_matches_individual_operations() {
        let mut bitmap = super::FlexBitMap::new();

        let test_cases = [
            (0, 1),     // single bit at start
            (10, 5),    // small range in middle
            (0, 64),    // exactly one block
            (0, 65),    // spans two blocks
            (100, 100), // arbitrary range
            (200, 56),  // range at end
            (0, 256),   // entire bitmap
            (128, 128), // second half
            (50, 150),  // large range spanning multiple blocks
        ];

        for (start, len) in test_cases {
            // Test with unset bits
            assert_eq!(
                bitmap.check_range_is_unset(start, len),
                check_range_all_unset(&bitmap, start, len),
                "Range check failed for unset bits ({start}, {len})"
            );

            // Set bits in the range
            set_bits_individually(&mut bitmap, start, len);

            // Test with set bits
            assert_eq!(
                bitmap.check_range_is_unset(start, len),
                check_range_all_unset(&bitmap, start, len),
                "Range check failed for set bits ({start}, {len})"
            );

            // Clear bits for next test
            clear_bits_individually(&mut bitmap, start, len);
        }
    }

    #[test]
    fn test_iter_range_matches_individual_operations() {
        let mut bitmap = super::FlexBitMap::new();

        let test_cases = [
            (0, 1),     // single bit at start
            (10, 5),    // small range in middle
            (0, 64),    // exactly one block
            (0, 65),    // spans two blocks
            (100, 100), // arbitrary range
            (200, 56),  // range at end
            (0, 256),   // entire bitmap
            (128, 128), // second half
            (50, 150),  // large range spanning multiple blocks
        ];

        for (start, len) in test_cases {
            // Test with unset bits
            let iter_result: Vec<(usize, bool)> = bitmap.iter_range(start, len).collect();
            let individual_result: Vec<(usize, bool)> =
                check_bits_individually(&bitmap, start, len)
                    .into_iter()
                    .enumerate()
                    .map(|(i, bit)| (start + i, bit))
                    .collect();

            assert_eq!(
                iter_result, individual_result,
                "Iterator mismatch for unset bits ({start}, {len})"
            );

            // Set some bits in the range
            for i in (0..len).step_by(3) {
                bitmap.set(start + i);
            }

            // Test with mixed bits
            let iter_result: Vec<(usize, bool)> = bitmap.iter_range(start, len).collect();
            let individual_result: Vec<(usize, bool)> =
                check_bits_individually(&bitmap, start, len)
                    .into_iter()
                    .enumerate()
                    .map(|(i, bit)| (start + i, bit))
                    .collect();

            assert_eq!(
                iter_result, individual_result,
                "Iterator mismatch for mixed bits ({start}, {len})"
            );

            // Clear bits for next test
            clear_bits_individually(&mut bitmap, start, len);
        }
    }

    #[test]
    fn test_edge_cases() {
        let mut bitmap = super::FlexBitMap::new();

        // Test operations beyond current capacity
        assert!(!bitmap.get(1000));
        bitmap.clear(1000); // Should not panic
        assert!(!bitmap.check_range_is_set(1000, 10));
        assert!(bitmap.check_range_is_unset(1000, 10));

        // Test zero-length operations
        bitmap.set_range(100, 0);
        bitmap.clear_range(100, 0);
        assert!(bitmap.check_range_is_set(100, 0));
        assert!(bitmap.check_range_is_unset(100, 0));

        let boundary_tests = [
            (0, 1),   // first bit
            (63, 1),  // end of first block
            (64, 1),  // start of second block
            (62, 4),  // spans block boundary
            (127, 1), // end of second block
            (128, 1), // start of third block
            (126, 4), // spans another block boundary
        ];

        for (start, len) in boundary_tests {
            // Test set_range
            bitmap.set_range(start, len);

            for i in start..start + len {
                assert!(bitmap.get(i), "Bit {i} should be set");
            }

            // Test clear_range
            bitmap.clear_range(start, len);

            for i in start..start + len {
                assert!(!bitmap.get(i), "Bit {i} should be clear");
            }
        }
    }

    #[test]
    fn test_count_operations_after_range_operations() {
        let mut bitmap = super::FlexBitMap::new();

        // Test counting after set_range
        bitmap.set_range(10, 20);
        assert_eq!(bitmap.count_ones(), 20);
        // count_zeros counts all zeros in the allocated blocks
        assert_eq!(bitmap.count_zeros(), bitmap.capacity() - 20);

        // Test counting after clear_range
        bitmap.clear_range(15, 10);
        assert_eq!(bitmap.count_ones(), 10); // Bits 10-14 are still set
        assert_eq!(bitmap.count_zeros(), bitmap.capacity() - 10);
    }

    #[test]
    fn test_range_operations_with_partial_patterns() {
        let mut bitmap = super::FlexBitMap::new();

        // Create a pattern: set every other bit
        for i in (0..100).step_by(2) {
            bitmap.set(i);
        }

        // Test that range operations work correctly with existing patterns
        bitmap.set_range(25, 10); // Should set bits 25-34
        for i in 25..35 {
            assert!(bitmap.get(i), "Bit {i} should be set after set_range");
        }

        bitmap.clear_range(30, 10); // Should clear bits 30-39
        for i in 30..40 {
            assert!(!bitmap.get(i), "Bit {i} should be clear after clear_range");
        }

        // Verify pattern outside ranges is preserved
        for i in (0..25).step_by(2) {
            assert!(bitmap.get(i), "Pattern bit {i} should be preserved");
        }
        for i in (40..100).step_by(2) {
            assert!(bitmap.get(i), "Pattern bit {i} should be preserved");
        }
    }

    #[test]
    fn test_capacity_and_blocks() {
        let mut bitmap = super::FlexBitMap::new();
        assert_eq!(bitmap.capacity(), 0);
        assert_eq!(bitmap.blocks(), 0);

        bitmap.set(10);
        assert_eq!(bitmap.blocks(), 1);
        assert!(bitmap.capacity() >= 11);

        bitmap.set(100);
        assert_eq!(bitmap.blocks(), 2);
        assert!(bitmap.capacity() >= 101);

        bitmap.set(200);
        assert_eq!(bitmap.blocks(), 4);
        assert!(bitmap.capacity() >= 201);
    }
}
