//! Fixed-size bitmap implementation.
//!
//! This module provides the main `FixedBitMap` type which offers a safe,
//! high-level interface for bitmap operations with compile-time size guarantees.
//! The implementation uses stack-allocated storage and provides
//! operations for individual bits and optimized operations for ranges.
//!
//! # Performance Characteristics
//!
//! - Individual bit operations: O(1) with single memory access
//! - Range operations: O(n) where n is the number of blocks affected
//! - Memory usage: Fixed at compile time (`BLOCKS * usize::BITS`)
//! - No heap allocation required
//!
//! # Safety
//!
//! All operations are bound-checked in debug builds. In release builds,
//! bound checking is omitted for maximum performance while maintaining
//! memory safety through the type system.

use crate::base;

/// A fixed-size bitmap with range operations.
///
/// This is the main type of the library, providing a stack-allocated bitmap
/// with a size determined at compile time. It offers efficient operations
/// for both individual bits and ranges of bits.
///
/// The bitmap is stored as an array of `usize` blocks, where each block
/// contains `usize::BITS` bits. For example, on a 64-bit system,
/// `FixedBitMap<4>` provides 256 bits of storage.
///
/// # Type Parameters
///
/// * `BLOCKS` - The number of `usize` blocks in the bitmap
///
/// # Examples
///
/// ```rust
/// use ranged_bitmap::FixedBitMap;
///
/// const BLOCKS: usize = ranged_bitmap::blocks_number_for_bits(100);
///
/// let mut bitmap = FixedBitMap::<BLOCKS>::new();
///
/// // Set bits individually
/// bitmap.set(0);
/// bitmap.set(99);
///
/// // Set a range efficiently
/// bitmap.set_range(10, 50);
///
/// assert!(bitmap.get(0));
/// assert!(bitmap.get(99));
/// assert!(bitmap.check_range_is_set(10, 50));
///
/// // Count set bits
/// assert_eq!(bitmap.count_ones(), 52); // 2 individual + 50 range
/// ```
#[derive(Clone)]
pub struct FixedBitMap<const BLOCKS: usize> {
    blocks: [usize; BLOCKS],
}

impl<const BLOCKS: usize> FixedBitMap<BLOCKS> {
    /// Create a new bitmap with all bits initially cleared.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ranged_bitmap::FixedBitMap;
    ///
    /// const BLOCKS: usize = ranged_bitmap::blocks_number_for_bits(100);
    ///
    /// let mut bitmap = FixedBitMap::<BLOCKS>::new();
    ///
    /// assert_eq!(bitmap.count_ones(), 0);
    /// assert_eq!(bitmap.count_zeros(), 128); // 2 * 64 bits on 64-bit system
    /// ```
    pub const fn new() -> Self {
        Self {
            blocks: [0; BLOCKS],
        }
    }

    /// Set a single bit to true.
    ///
    /// This function sets the specified bit to `true` using a single bitwise
    /// OR operation.
    ///
    /// # Panics
    ///
    /// In debug builds, this function will panic if `bit` is out of bounds
    /// (i.e., `bit >= BLOCKS * usize::BITS`). In release builds, bound
    /// checking is omitted for performance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ranged_bitmap::FixedBitMap;
    ///
    /// let mut bitmap = FixedBitMap::<1>::new();
    ///
    /// bitmap.set(10);
    /// bitmap.set(30);
    ///
    /// assert!(bitmap.get(10));
    /// assert!(bitmap.get(30));
    /// ```
    #[inline(always)]
    pub const fn set(&mut self, bit: usize) {
        base::bitmap_set(&mut self.blocks, bit);
    }

    /// Clear a single bit to false.
    ///
    /// This function sets the specified bit to `false` using a single bitwise
    /// AND operation with an inverted mask.
    ///
    /// # Panics
    ///
    /// In debug builds, this function will panic if `bit` is out of bounds.
    /// In release builds, bound checking is omitted for performance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ranged_bitmap::FixedBitMap;
    ///
    /// let mut bitmap = FixedBitMap::<1>::new();
    ///
    /// bitmap.set_range(0, 32); // Set all bits from 0 to 31
    /// bitmap.clear(10);
    ///
    /// assert!(!bitmap.get(10));
    /// assert!(bitmap.get(11));
    /// ```
    #[inline(always)]
    pub const fn clear(&mut self, bit: usize) {
        base::bitmap_clear(&mut self.blocks, bit);
    }

    /// Get the value of a single bit.
    ///
    /// This function returns `true` if the specified bit is set, `false` otherwise.
    ///
    /// # Panics
    ///
    /// In debug builds, this function will panic if `bit` is out of bounds.
    /// In release builds, bound checking is omitted for performance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ranged_bitmap::FixedBitMap;
    ///
    /// let mut bitmap = FixedBitMap::<1>::new();
    ///
    /// bitmap.set(10);
    ///
    /// assert!(bitmap.get(10));
    /// assert!(!bitmap.get(11));
    /// ```
    #[inline(always)]
    pub const fn get(&self, bit: usize) -> bool {
        base::bitmap_get(&self.blocks, bit)
    }

    /// Count the number of set bits (ones) in the bitmap.
    ///
    /// This function iterates over all blocks and uses the hardware-accelerated
    /// `count_ones()` method to count set bits efficiently. The operation is
    /// `O(n)` where n is the number of blocks.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ranged_bitmap::FixedBitMap;
    ///
    /// let mut bitmap = FixedBitMap::<2>::new();
    ///
    /// bitmap.set(10);
    /// bitmap.set(70);
    /// bitmap.set(127);
    ///
    /// assert_eq!(bitmap.count_ones(), 3);
    /// ```
    pub const fn count_ones(&self) -> usize {
        base::bitmap_count_ones(&self.blocks)
    }

    /// Count the number of clear bits (zeros) in the bitmap.
    ///
    /// This function iterates over all blocks and uses the hardware-accelerated
    /// `count_zeros()` method to count clear bits efficiently. The operation is
    /// `O(n)` where n is the number of blocks.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ranged_bitmap::FixedBitMap;
    ///
    /// let mut bitmap = FixedBitMap::<1>::new();
    ///
    /// assert_eq!(bitmap.count_zeros(), usize::BITS as usize);
    ///
    /// bitmap.set(10);
    /// assert_eq!(bitmap.count_zeros(), usize::BITS as usize - 1); // One bit is set
    /// ```
    pub const fn count_zeros(&self) -> usize {
        base::bitmap_count_zeros(&self.blocks)
    }

    /// Create an iterator over a range of bits.
    ///
    /// This function returns an iterator that yields tuples of `(index, value)`
    /// for each bit in the specified range. The iterator is exact-sized and
    /// provides efficient range traversal.
    ///
    /// # Panics
    ///
    /// In debug builds, this function will panic if the range is out of bounds.
    /// In release builds, bound checking is omitted for performance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ranged_bitmap::FixedBitMap;
    ///
    /// let mut bitmap = FixedBitMap::<1>::new();
    ///
    /// bitmap.set(10);
    /// bitmap.set(12);
    /// bitmap.set(15);
    ///
    /// let bits: Vec<(usize, bool)> = bitmap.iter_range(8, 10).collect();
    /// assert_eq!(bits, vec![(8, false), (9, false), (10, true), (11, false), (12, true), (13, false), (14, false), (15, true), (16, false), (17, false)]);
    /// ```
    pub const fn iter_range(
        &self,
        start: usize,
        len: usize,
    ) -> impl Iterator<Item = (usize, bool)> + use<'_, BLOCKS> {
        base::bitmap_iter_range(&self.blocks, start, len)
    }

    /// Set a range of bits to true.
    ///
    /// This function sets all bits in the specified range to `true` using
    /// optimized block operations. The operation is much more efficient than
    /// setting bits individually, especially for large ranges.
    ///
    /// # Panics
    ///
    /// In debug builds, this function will panic if the range is out of bounds.
    /// In release builds, bound checking is omitted for performance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ranged_bitmap::FixedBitMap;
    ///
    /// let mut bitmap = FixedBitMap::<2>::new();
    ///
    /// bitmap.set_range(10, 50);  // Set bits 10-59
    ///
    /// for i in 10..60 {
    ///     assert!(bitmap.get(i));
    /// }
    ///
    /// assert!(!bitmap.get(9));   // Before range is not set
    /// assert!(!bitmap.get(60));  // After range is not set
    /// ```
    pub const fn set_range(&mut self, start: usize, len: usize) {
        base::bitmap_set_range(&mut self.blocks, start, len);
    }

    /// Clear a range of bits to false.
    ///
    /// This function clears all bits in the specified range to `false` using
    /// optimized block operations. The operation is much more efficient than
    /// clearing bits individually, especially for large ranges.
    ///
    /// # Panics
    ///
    /// In debug builds, this function will panic if the range is out of bounds.
    /// In release builds, bound checking is omitted for performance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ranged_bitmap::FixedBitMap;
    ///
    /// let mut bitmap = FixedBitMap::<2>::new();
    ///
    /// bitmap.set_range(0, usize::BITS as usize * 2);  // Set all bits
    /// bitmap.clear_range(10, 50); // Clear bits 10-59
    ///
    /// for i in 10..60 {
    ///     assert!(!bitmap.get(i));
    /// }
    ///
    /// assert!(bitmap.get(9));   // Before range remains set
    /// assert!(bitmap.get(60));  // After range remains set
    /// ```
    pub const fn clear_range(&mut self, start: usize, len: usize) {
        base::bitmap_clear_range(&mut self.blocks, start, len);
    }

    /// Check if all bits in a range are set.
    ///
    /// This function returns `true` if and only if every bit in the specified
    /// range is set to `true`. The operation is optimized using precomputed
    /// lookup tables and early termination for large ranges.
    ///
    /// # Panics
    ///
    /// In debug builds, this function will panic if the range is out of bounds.
    /// In release builds, bound checking is omitted for performance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ranged_bitmap::FixedBitMap;
    ///
    /// let mut bitmap = FixedBitMap::<2>::new();
    ///
    /// // Initially no bits are set
    /// assert!(!bitmap.check_range_is_set(10, 50));
    ///
    /// // Set the entire range
    /// bitmap.set_range(10, 50);
    /// assert!(bitmap.check_range_is_set(10, 50));
    /// ```
    pub const fn check_range_is_set(&self, start: usize, len: usize) -> bool {
        base::bitmap_check_range_is_set(&self.blocks, start, len)
    }

    /// Check if all bits in a range are unset.
    ///
    /// This function returns `true` if and only if every bit in the specified
    /// range is set to `false`. The operation is optimized using precomputed
    /// lookup tables and early termination for large ranges.
    ///
    /// # Panics
    ///
    /// In debug builds, this function will panic if the range is out of bounds.
    /// In release builds, bound checking is omitted for performance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ranged_bitmap::FixedBitMap;
    ///
    /// let mut bitmap = FixedBitMap::<2>::new();
    ///
    /// // Initially all bits are unset
    /// assert!(bitmap.check_range_is_unset(10, 50));
    ///
    /// // Set some bits in the range
    /// bitmap.set_range(10, 25);  // Set first half
    /// assert!(!bitmap.check_range_is_unset(10, 50)); // No longer all unset
    ///
    /// // Clear the range again
    /// let mut bitmap = FixedBitMap::<2>::new();
    /// assert!(bitmap.check_range_is_unset(10, 50)); // All unset again
    /// ```
    pub const fn check_range_is_unset(&self, start: usize, len: usize) -> bool {
        base::bitmap_check_range_is_unset(&self.blocks, start, len)
    }

    /// Find the first set bit starting from the specified offset.
    ///
    /// Searches the bitmap beginning at `start` (**inclusive**) and returns the
    /// index of the first bit whose value is `true`.
    ///
    /// If no set bit exists at or after `start`, this function returns `None`.
    ///
    /// The search is optimized to process words, not individual bits,
    /// making it fast enough for expected sizes of bit maps.
    ///
    /// # Panics
    ///
    /// In debug builds, this function will panic if `start` is out of bounds.
    /// In release builds, bound checking is omitted for performance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ranged_bitmap::FixedBitMap;
    ///
    /// let mut bitmap = FixedBitMap::<8>::new();
    ///
    /// bitmap.set(0);
    /// bitmap.set(1);
    /// bitmap.set(10);
    /// bitmap.set(42);
    /// bitmap.set(80);
    /// bitmap.set(300);
    ///
    /// assert_eq!(bitmap.first_set_bit_from_offset(0), Some(0));
    /// assert_eq!(bitmap.first_set_bit_from_offset(1), Some(1));
    /// assert_eq!(bitmap.first_set_bit_from_offset(2), Some(10));
    /// assert_eq!(bitmap.first_set_bit_from_offset(11), Some(42));
    /// assert_eq!(bitmap.first_set_bit_from_offset(43), Some(80));
    /// assert_eq!(bitmap.first_set_bit_from_offset(81), Some(300));
    /// assert_eq!(bitmap.first_set_bit_from_offset(300), Some(300));
    /// assert_eq!(bitmap.first_set_bit_from_offset(301), None);
    /// ```
    pub const fn first_set_bit_from_offset(&self, start: usize) -> Option<usize> {
        base::first_set_bit_from_offset(&self.blocks, start)
    }

    /// Find the first unset bit starting from the specified offset.
    ///
    /// Searches the bitmap beginning at `start` (**inclusive**) and returns the
    /// index of the first bit whose value is `false`.
    ///
    /// If no unset bit exists at or after `start`, this function returns `None`.
    ///
    /// The search is optimized to process words, not individual bits,
    /// making it fast enough for expected sizes of bit maps.
    ///
    /// # Panics
    ///
    /// In debug builds, this function will panic if `start` is out of bounds.
    /// In release builds, bound checking is omitted for performance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ranged_bitmap::FixedBitMap;
    ///
    /// let mut bitmap = FixedBitMap::<8>::new();
    /// bitmap.set_range(0, 8 * usize::BITS as usize);
    ///
    /// bitmap.clear(0);
    /// bitmap.clear(1);
    /// bitmap.clear(10);
    /// bitmap.clear(42);
    /// bitmap.clear(80);
    /// bitmap.clear(300);
    ///
    /// assert_eq!(bitmap.first_unset_bit_from_offset(0), Some(0));
    /// assert_eq!(bitmap.first_unset_bit_from_offset(1), Some(1));
    /// assert_eq!(bitmap.first_unset_bit_from_offset(2), Some(10));
    /// assert_eq!(bitmap.first_unset_bit_from_offset(11), Some(42));
    /// assert_eq!(bitmap.first_unset_bit_from_offset(43), Some(80));
    /// assert_eq!(bitmap.first_unset_bit_from_offset(81), Some(300));
    /// assert_eq!(bitmap.first_unset_bit_from_offset(300), Some(300));
    /// assert_eq!(bitmap.first_unset_bit_from_offset(301), None);
    /// ```
    pub const fn first_unset_bit_from_offset(&self, start: usize) -> Option<usize> {
        base::first_unset_bit_from_offset(&self.blocks, start)
    }
}

impl<const BLOCKS: usize> Default for FixedBitMap<BLOCKS> {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a typed bitmap wrapper struct.
///
/// This macro creates a new struct that wraps a `FixedBitMap` with a specific
/// number of bits. The generated struct implements `Deref` and `DerefMut`
/// to provide transparent access to the underlying [`FixedBitMap`] methods.
///
/// This is useful for creating type-safe bitmap wrappers with compile-time
/// size guarantees.
///
/// # Syntax
///
/// ```rust
/// use ranged_bitmap::generate_fixed_bit_map_struct;
///
/// generate_fixed_bit_map_struct!(pub(crate) struct StructName<128>);
/// ```
///
/// # Arguments
///
/// * `$vis:vis` - Visibility modifier for the generated struct, or `pub(self)` by default.
/// * `struct $name:ident` - The name of the struct to generate
/// * `<$bits:tt>` - The number of bits the bitmap should contain
///
/// # Generated Code
///
/// The macro generates:
/// - A struct wrapping `FixedBitMap<BLOCKS>` where `BLOCKS` is computed from `bits`
/// - A `new()` constructor function
/// - `Deref` and `DerefMut` implementations for transparent method access
///
/// # Example
///
/// ```rust
/// use ranged_bitmap::generate_fixed_bit_map_struct;
///
/// generate_fixed_bit_map_struct!(pub(crate) struct MyBitmap<256>);
///
/// let mut bitmap = MyBitmap::new();
/// bitmap.set(10);  // Uses Deref to access FixedBitMap methods
/// bitmap.set_range(50, 100);
///
/// assert!(bitmap.get(10));
/// assert!(bitmap.check_range_is_set(50, 100));
/// ```
///
/// # Performance
///
/// The generated wrapper has zero runtime overhead: all method calls
/// are directly forwarded to the underlying `FixedBitMap` through
/// deref coercion, which is optimized away by the compiler.
#[macro_export]
macro_rules! generate_fixed_bit_map_struct {
    ($vis:vis struct $name:ident<$bits:tt>) => {
        $vis const __BLOCKS: usize = $crate::blocks_number_for_bits($bits);

        #[derive(Clone)]
        $vis struct $name($vis $crate::FixedBitMap<__BLOCKS>);

        impl $name {
            $vis const fn new() -> Self {
                Self($crate::FixedBitMap::new())
            }
        }

        impl core::ops::Deref for $name {
            type Target = $crate::FixedBitMap<__BLOCKS>;

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

    (struct $name:ident<$bits:tt>) => {
        generate_fixed_bit_map_struct!(pub(self), struct $name<$bits>)
    };
}