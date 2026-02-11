//! Base module containing core bitmap operations and utilities.
//!
//! This module provides low-level bitmap operations that serve
//! as the foundation for the higher-level `FixedBitMap` interface. All operations
//! are optimized for performance with aggressive inlining and precomputed
//! lookup tables.
//!
//! # Performance Characteristics
//!
//! - Individual bit operations: O(1) with single memory access
//! - Range operations: O(n) where n is the number of blocks affected
//! - Full block operations use memset-like optimizations
//! - All functions are const and marked `#[inline(always)]`
//!
//! # Safety
//!
//! All functions perform bound checking in debug builds. In release builds,
//! bounds are elided for maximum performance. Callers must ensure valid indices.

use crate::maybe_assert;

/// Calculate the number of blocks needed to store a given number of bits.
/// 
/// This function computes the minimum number of `usize` blocks required to
/// store the specified number of bits, rounding up when necessary. This is
/// a constant function that can be evaluated at compile time.
///
/// 
/// # Example
/// 
/// ```rust
/// use ranged_bitmap::blocks_number_for_bits;
/// 
/// const BLOCKS_64: usize = blocks_number_for_bits(64);
/// const BLOCKS_65: usize = blocks_number_for_bits(65);
/// 
/// assert_eq!(BLOCKS_64, 1);  // Exactly one block
/// assert_eq!(BLOCKS_65, 2);  // Needs two blocks
/// ```
#[inline(always)]
pub const fn blocks_number_for_bits(requested_bits: usize) -> usize {
    (requested_bits / usize::BITS as usize)
        + if requested_bits % usize::BITS as usize != 0 {
            1
        } else {
            0
        }
}

/// Calculate the block index and slot position for a given bit.
/// 
/// This function converts a global bit index into the corresponding block
/// index and the position within that block.
#[inline(always)]
const fn calc_block_and_slot(bit: usize) -> (usize, usize) {
    (bit / usize::BITS as usize, bit % usize::BITS as usize)
}

/// Set a single bit to true.
/// 
/// # Example
/// 
/// ```rust
/// use ranged_bitmap::base::bitmap_set;
///
/// let mut blocks = [0usize; 2];
///
/// bitmap_set(&mut blocks, 10);
/// bitmap_set(&mut blocks, 70);
///
/// assert!(blocks[0] & (1 << 10) != 0);  // Bit 10 is set
/// assert!(blocks[1] & (1 << 6) != 0);   // Bit 70 is set (block 1, slot 6)
/// ```
#[inline(always)]
pub const fn bitmap_set(blocks: &mut [usize], bit: usize) {
    maybe_assert(
        bit < blocks.len() * usize::BITS as usize,
        "`bit` is out of bounds",
    );

    let (block, slot) = calc_block_and_slot(bit);

    blocks[block] |= 1 << slot;
}

/// Clear a single bit to false.
/// 
/// This function sets the specified bit to `false`.
/// The operation is performed in with bounds checking in debug builds.
///
/// 
/// # Safety
/// 
/// In debug builds, this function will panic if `bit` is out of bounds.
/// In release builds, bound checking is omitted for performance.
/// 
/// # Example
/// 
/// ```rust
/// use ranged_bitmap::base::{bitmap_set, bitmap_clear};
///
/// let mut blocks = [usize::MAX; 1];
///
/// bitmap_clear(&mut blocks, 10);
///
/// assert!(blocks[0] & (1 << 10) == 0);  // Bit 10 is cleared
/// assert!(blocks[0] & (1 << 11) != 0);  // Other bits remain set
/// ```
#[inline(always)]
pub const fn bitmap_clear(blocks: &mut [usize], bit: usize) {
    maybe_assert(
        bit < blocks.len() * usize::BITS as usize,
        "`bit` is out of bounds",
    );

    let (block, slot) = calc_block_and_slot(bit);

    blocks[block] &= !(1 << slot);
}

/// Get the value of a single bit.
/// 
/// This function returns `true` if the specified bit is set, `false` otherwise.
/// The operation is performed checking in debug builds.
/// 
/// # Safety
/// 
/// In debug builds, this function will panic if `bit` is out of bounds.
/// In release builds, bound checking is omitted for performance.
/// 
/// # Example
/// 
/// ```rust
/// use ranged_bitmap::base::{bitmap_set, bitmap_get};
///
/// let mut blocks = [0usize; 1];
///
/// bitmap_set(&mut blocks, 10);
///
/// assert!(bitmap_get(&blocks, 10));   // Bit 10 is set
/// assert!(!bitmap_get(&blocks, 11));  // Bit 11 is not set
/// ```
#[inline(always)]
pub const fn bitmap_get(blocks: &[usize], bit: usize) -> bool {
    maybe_assert(
        bit < blocks.len() * usize::BITS as usize,
        "`bit` is out of bounds",
    );

    let (block, slot) = calc_block_and_slot(bit);

    blocks[block] & (1 << slot) != 0
}

/// Count the number of set bits (ones) in the bitmap.
/// 
/// This function iterates over all blocks and uses the hardware-accelerated
/// `count_ones()` method to count set bits efficiently. The operation is
/// O(n) where n is the number of blocks.
/// 
/// # Example
/// 
/// ```rust
/// use ranged_bitmap::base::{bitmap_set, bitmap_count_ones};
///
/// let mut blocks = [0usize; 2];
///
/// bitmap_set(&mut blocks, 10);
/// bitmap_set(&mut blocks, 70);
/// bitmap_set(&mut blocks, 127);
///
/// assert_eq!(bitmap_count_ones(&blocks), 3);
/// ```
pub const fn bitmap_count_ones(blocks: &[usize]) -> usize {
    let mut count = 0;
    let mut i = 0;

    while i < blocks.len() {
        count += blocks[i].count_ones() as usize;

        i += 1;
    }

    count
}

/// Count the number of clear bits (zeros) in the bitmap.
/// 
/// This function iterates over all blocks and uses the hardware-accelerated
/// `count_zeros()` method to count clear bits efficiently. The operation is
/// O(n) where n is the number of blocks.
///
/// 
/// # Example
/// 
/// ```rust
/// use ranged_bitmap::base::{bitmap_set, bitmap_count_zeros};
///
/// let mut blocks = [usize::MAX; 1];  // All bits set
/// bitmap_set(&mut blocks, 10);               // Set bit 10 (already set)
///
/// assert_eq!(bitmap_count_zeros(&blocks), 0); // No clear bits
///
/// let mut blocks = [0usize; 1];       // All bits clear
/// assert_eq!(bitmap_count_zeros(&blocks), 64); // 64 clear bits on 64-bit system
/// ```
pub const fn bitmap_count_zeros(blocks: &[usize]) -> usize {
    let mut count = 0;
    let mut i = 0;

    while i < blocks.len() {
        count += blocks[i].count_zeros() as usize;

        i += 1;
    }

    count
}

/// Create an iterator over a range of bits.
/// 
/// This function returns an iterator that yields tuples of `(index, value)`
/// for each bit in the specified range. The iterator is exact-sized and
/// provides efficient range traversal.
///
/// 
/// # Example
/// 
/// ```rust
/// use ranged_bitmap::base::{bitmap_set, bitmap_iter_range};
///
/// let mut blocks = [0usize; 1];
///
/// bitmap_set(&mut blocks, 10);
/// bitmap_set(&mut blocks, 12);
/// bitmap_set(&mut blocks, 15);
///
/// let bits: Vec<(usize, bool)> = bitmap_iter_range(&blocks, 8, 10).collect();
/// assert_eq!(bits, vec![(8, false), (9, false), (10, true), (11, false), (12, true), (13, false), (14, false), (15, true), (16, false), (17, false)]);
/// ```
pub const fn bitmap_iter_range(
    blocks: &[usize],
    start: usize,
    len: usize,
) -> impl Iterator<Item = (usize, bool)> + use<'_> {
    struct Iter<'bitmap> {
        bitmap: &'bitmap [usize],
        curr: usize,
        end: usize,
    }

    impl Iterator for Iter<'_> {
        type Item = (usize, bool);

        fn next(&mut self) -> Option<Self::Item> {
            if self.curr < self.end {
                let res = Some((self.curr, bitmap_get(self.bitmap, self.curr)));

                self.curr += 1;

                return res;
            }

            None
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.end - self.curr, Some(self.end - self.curr))
        }
    }

    impl ExactSizeIterator for Iter<'_> {
        fn len(&self) -> usize {
            self.end - self.curr
        }
    }

    Iter {
        bitmap: blocks,
        curr: start,
        end: start + len,
    }
}

/// Get a reference to an array element by index.
/// 
/// This function provides optimized array access. In test builds, it uses
/// normal indexing for safety. In release builds, it uses unsafe pointer
/// arithmetic for maximum performance while maintaining correctness.
/// 
/// # Safety
/// 
/// In release builds, this function performs unchecked access. Callers must
/// ensure the index is within bounds.
const fn get_ref_by_index<T>(arr: &[T], idx: usize) -> &T {
    if cfg!(test) {
        &arr[idx]
    } else {
        unsafe { &*arr.as_ptr().add(idx) }
    }
}

/// Get a mutable reference to an array element by index.
/// 
/// This function provides optimized mutable array access. In test builds,
/// it uses normal indexing for safety. In release builds, it uses unsafe
/// pointer arithmetic for maximum performance while maintaining correctness.
/// 
/// # Safety
/// 
/// In release builds, this function performs unchecked access. Callers must
/// ensure the index is within bounds.
const fn get_mut_by_index<T>(arr: &mut [T], idx: usize) -> &mut T {
    if cfg!(test) {
        &mut arr[idx]
    } else {
        unsafe { &mut *arr.as_mut_ptr().add(idx) }
    }
}

/// Update a range of bits using optimized block operations.
/// 
/// The function uses precomputed lookup tables for optimal performance and
/// can perform both set and clear operations based on the `is_set_operation` flag.
/// 
/// # Safety
/// 
/// In debug builds, this function will panic if the range is out of bounds.
/// In release builds, bound checking is omitted for performance.
const fn update_range(
    blocks: &mut [usize],
    start: usize,
    len: usize,
    set_bits_for_len: &[usize; usize::BITS as usize + 1],
    is_set_operation: bool,
) {
    maybe_assert(
        start + len <= blocks.len() * usize::BITS as usize,
        "`start` + `len` is out of bounds",
    );

    // Three cases are possible:
    // 1. We need to update only one block;
    // 2. We need to update only two blocks;
    // 3. We need to update more than two blocks.
    // It looks obvious, like why did I write this?
    // Because each of these cases has its own algorithm.

    let (block, start_bit) = calc_block_and_slot(start);
    let end = start_bit + len;

    if end <= usize::BITS as usize {
        // one block
        let mask = *get_ref_by_index(set_bits_for_len, len) << start_bit;

        if is_set_operation {
            *get_mut_by_index(blocks, block) |= mask;
        } else {
            *get_mut_by_index(blocks, block) &= !mask;
        }
    } else if end <= (usize::BITS * 2) as usize {
        // two blocks
        let first_block_len = usize::BITS as usize - start_bit;
        let second_block_len = len - first_block_len;
        let first_mask = *get_ref_by_index(set_bits_for_len, first_block_len) << start_bit;
        let second_mask = *get_ref_by_index(set_bits_for_len, second_block_len);

        if is_set_operation {
            *get_mut_by_index(blocks, block) |= first_mask;
            *get_mut_by_index(blocks, block + 1) |= second_mask;
        } else {
            *get_mut_by_index(blocks, block) &= !first_mask;
            *get_mut_by_index(blocks, block + 1) &= !second_mask;
        }
    } else {
        // many blocks
        let first_block_len = usize::BITS as usize - start_bit;
        let remaining = len - first_block_len;
        let number_of_full_blocks = remaining / usize::BITS as usize;
        let last_block_len = remaining % usize::BITS as usize;
        let first_mask = *get_ref_by_index(set_bits_for_len, first_block_len) << start_bit;

        if is_set_operation {
            *get_mut_by_index(blocks, block) |= first_mask;

            if last_block_len > 0 {
                let second_mask = *get_ref_by_index(set_bits_for_len, last_block_len);

                *get_mut_by_index(blocks, block + number_of_full_blocks + 1) |= second_mask;
            }

            unsafe {
                blocks
                    .as_mut_ptr()
                    .add(block + 1)
                    .write_bytes(255, number_of_full_blocks);
            }
        } else {
            *get_mut_by_index(blocks, block) &= !first_mask;

            if last_block_len > 0 {
                let second_mask = *get_ref_by_index(set_bits_for_len, last_block_len);

                *get_mut_by_index(blocks, block + number_of_full_blocks + 1) &= !second_mask;
            }

            unsafe {
                blocks
                    .as_mut_ptr()
                    .add(block + 1)
                    .write_bytes(0, number_of_full_blocks);
            }
        }
    }
}

/// Precomputed lookup table for setting bits of a given length.
///
/// 
/// # Example
/// 
/// ```text
/// // On a 64-bit system:
/// assert_eq!(SET_BITS_FOR_LEN[0], 0);          // No bits are set
/// assert_eq!(SET_BITS_FOR_LEN[1], 1);          // First bit is set
/// assert_eq!(SET_BITS_FOR_LEN[3], 0b111);      // First 3 bits are set
/// assert_eq!(SET_BITS_FOR_LEN[64], usize::MAX); // All bits are set
/// ```
const SET_BITS_FOR_LEN: [usize; usize::BITS as usize + 1] = {
    let mut res = [0; usize::BITS as usize + 1];
    let mut curr = 0;

    while curr < usize::BITS as usize {
        let mut i = 0;
        let mut block = 0;

        while i <= curr {
            block |= 1 << i;

            i += 1;
        }

        res[curr + 1] = block;

        curr += 1;
    }

    res
};

/// Set a range of bits to true.
/// 
/// This function sets all bits in the specified range to `true` using
/// optimized block operations. The operation is much more efficient than
/// setting bits individually, especially for large ranges.
///
/// 
/// # Example
/// 
/// ```rust
/// use ranged_bitmap::base::{bitmap_set_range, bitmap_get};
///
/// let mut blocks = [0usize; 2];
/// bitmap_set_range(&mut blocks, 10, 50);  // Set bits 10-59
///
/// for i in 10..60 {
///     assert!(bitmap_get(&blocks, i));
/// }
///
/// assert!(!bitmap_get(&blocks, 9));   // Before range is not set
/// assert!(!bitmap_get(&blocks, 60));  // After range is not set
/// ```
pub const fn bitmap_set_range(blocks: &mut [usize], start: usize, len: usize) {
    update_range(blocks, start, len, &SET_BITS_FOR_LEN, true);
}

/// Clear a range of bits to false.
/// 
/// This function clears all bits in the specified range to `false` using
/// optimized block operations. The operation is much more efficient than
/// clearing bits individually, especially for large ranges.
///
/// 
/// # Example
/// 
/// ```rust
/// use ranged_bitmap::base::{bitmap_clear_range, bitmap_set, bitmap_get};
///
/// let mut blocks = [usize::MAX; 2];  // All bits set
///
/// bitmap_clear_range(&mut blocks, 10, 50); // Clear bits 10-59
///
/// for i in 10..60 {
///     assert!(!bitmap_get(&blocks, i));
/// }
///
/// assert!(bitmap_get(&blocks, 9));   // Before range remains set
/// assert!(bitmap_get(&blocks, 60));  // After range remains set
/// ```
pub const fn bitmap_clear_range(blocks: &mut [usize], start: usize, len: usize) {
    update_range(blocks, start, len, &SET_BITS_FOR_LEN, false);
}

/// Internal function to check if a range meets a specific condition.
/// 
/// This function checks whether all bits in a range are either set or unset
/// based on the `is_set_check` parameter.
/// 
/// # Safety
/// 
/// In debug builds, this function will panic if the range is out of bounds.
/// In release builds, bound checking is omitted for performance.
const fn check_range_internal(
    blocks: &[usize],
    start: usize,
    len: usize,
    check_bits_for_len: &[usize; usize::BITS as usize + 1],
    is_set_check: bool,
) -> bool {
    maybe_assert(
        start + len <= blocks.len() * usize::BITS as usize,
        "`start` + `len` is out of bounds",
    );

    let mut ones = 0;

    // Three cases are possible:
    // 1. We need to check only one block;
    // 2. We need to check only two blocks;
    // 3. We need to check more than two blocks.
    // It looks obvious, like why did I write this?
    // Because each of these cases has its own algorithm.

    let (block, start_bit) = calc_block_and_slot(start);
    let end = start_bit + len;

    if end <= usize::BITS as usize {
        // one block
        let mask = *get_ref_by_index(check_bits_for_len, len) << start_bit;

        ones += (*get_ref_by_index(blocks, block) & mask).count_ones();
    } else if end <= (usize::BITS * 2) as usize {
        // two blocks
        let first_block_len = usize::BITS as usize - start_bit;
        let second_block_len = len - first_block_len;
        let first_mask = *get_ref_by_index(check_bits_for_len, first_block_len) << start_bit;
        let second_mask = *get_ref_by_index(check_bits_for_len, second_block_len);

        ones += (*get_ref_by_index(blocks, block) & first_mask).count_ones();
        ones += (*get_ref_by_index(blocks, block + 1) & second_mask).count_ones();
    } else {
        // many blocks
        let first_block_len = usize::BITS as usize - start_bit;
        let remaining = len - first_block_len;
        let number_of_full_blocks = remaining / usize::BITS as usize;
        let last_block_len = remaining % usize::BITS as usize;
        let first_mask = *get_ref_by_index(check_bits_for_len, first_block_len) << start_bit;

        ones += (*get_ref_by_index(blocks, block) & first_mask).count_ones();

        if last_block_len > 0 {
            let second_mask = *get_ref_by_index(check_bits_for_len, last_block_len);

            ones += (*get_ref_by_index(blocks, block + number_of_full_blocks + 1) & second_mask).count_ones();
        }

        let mut i = 1;
        let end = 1 + number_of_full_blocks;

        if len <= 4096 {
            while i < end {
                ones += get_ref_by_index(blocks, block + i).count_ones();

                i += 1;
            }
        } else {
            while i < end {
                let ones_here = get_ref_by_index(blocks, block + i).count_ones();
                if is_set_check && ones_here < usize::BITS || !is_set_check && ones_here > 0 {
                    return false;
                }

                ones += ones_here;

                i += 1;
            }
        }
    }

    (is_set_check && ones as usize == len) || (!is_set_check && ones == 0)
}

/// Check if all bits in a range are set.
/// 
/// This function returns `true` if and only if every bit in the specified
/// range is set to `true`. The operation is optimized using precomputed
/// lookup tables and early termination for large ranges.
/// 
/// # Example
/// 
/// ```rust
/// use ranged_bitmap::base::{bitmap_set_range, bitmap_check_range_is_set};
///
/// let mut blocks = [0usize; 2];
///
/// // Initially no bits are set
/// assert!(!bitmap_check_range_is_set(&blocks, 10, 50));
///
/// // Set the entire range
/// bitmap_set_range(&mut blocks, 10, 50);
/// assert!(bitmap_check_range_is_set(&blocks, 10, 50));
/// ```
pub const fn bitmap_check_range_is_set(blocks: &[usize], start: usize, len: usize) -> bool {
    check_range_internal(blocks, start, len, &SET_BITS_FOR_LEN, true)
}

/// Check if all bits in a range are unset.
/// 
/// This function returns `true` if and only if every bit in the specified
/// range is set to `false`. The operation is optimized using precomputed
/// lookup tables and early termination for large ranges.
/// 
/// # Performance
/// 
/// - Small ranges: O(n) where n is the number of affected blocks
/// - Large ranges: Early termination when a set bit is found
/// - Uses hardware-accelerated bit counting operations
/// 
/// # Example
/// 
/// ```rust
/// use ranged_bitmap::base::{bitmap_set_range, bitmap_check_range_is_unset};
///
/// let mut blocks = [0usize; 2];
///
/// // Initially all bits are unset
/// assert!(bitmap_check_range_is_unset(&blocks, 10, 50));
///
/// // Set some bits in the range
/// bitmap_set_range(&mut blocks, 10, 25);  // Set first half
/// assert!(!bitmap_check_range_is_unset(&blocks, 10, 50)); // No longer all unset
///
/// // Clear the range again
/// let mut blocks = [0usize; 2];
/// assert!(bitmap_check_range_is_unset(&blocks, 10, 50)); // All unset again
/// ```
pub const fn bitmap_check_range_is_unset(blocks: &[usize], start: usize, len: usize) -> bool {
    check_range_internal(blocks, start, len, &SET_BITS_FOR_LEN, false)
}
