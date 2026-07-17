#![no_std]

//! # Ranged Bitmap
//!
//! A high-performance bitmap library with fixed-size, flexible, and small-optimized bitmap implementations.
//!
//! This library provides efficient bitmap manipulation with a focus on:
//! - All functions are **constant**
//! - **Optimized range operations** that work on multiple bits simultaneously
//! - **Fixed-size bitmaps** with compile-time size determination
//! - **Flexible bitmaps** that grow dynamically as needed
//! - **Small-optimized bitmaps** using [`SmallVec`](smallvec::SmallVec) for stack
//!   storage with automatic heap growth
//! - **`No_std` compatibility** for embedded and bare-metal environments
//!
//! ## Features
//!
//! - Fixed-size bitmaps with compile-time size determination
//! - Flexible bitmaps that grow dynamically as needed
//! - Small-optimized bitmaps using [`SmallVec`](smallvec::SmallVec) for efficient storage
//! - Efficient range setting, clearing, and checking operations
//! - All functions are constant
//!
//! ## Performance
//!
//! The library is designed with performance as a primary goal:
//! - Range operations use precomputed lookup tables for optimal bit manipulation
//! - Bulk operations on full blocks use `memset`-like operations for maximum speed
//! - All functions are marked `#[inline(always)]` for aggressive inlining
//! - Constant functions enable compile-time optimizations
//! - Hardware-accelerated bit counting operations
//!
//! ## Bitmap Type Comparison
//!
//! | Type | Storage | Best For | Heap Allocation |
//! |------|---------|-----------|-----------------|
//! | `FixedBitMap` | Stack only | Known size at compile time | Never |
//! | `SmallBitMap` | Stack + [`SmallVec`](smallvec::SmallVec) | Usually small, occasionally large | Only when needed |
//! | `FlexBitMap` | Heap only | Unknown size, always dynamic | Always |
//!
//! ## Examples
//!
//! ### Fixed Bitmap (Compile-time size)
//!
//! ```rust
//! use ranged_bitmap::generate_fixed_bit_map_struct;
//!
//! generate_fixed_bit_map_struct!(struct BitMap<256>); // Generates a 256-bit bitmap struct wrapper
//!
//! // Create a 256-bit bitmap
//! let mut bitmap = BitMap::new();
//!
//! // Set individual bits
//! bitmap.set(10);
//! bitmap.set(20);
//!
//! // Gets individual bits
//! assert!(bitmap.get(10));
//! assert!(!bitmap.get(21));
//!
//! // Set an entire range at once (much faster than individual operations)
//! bitmap.set_range(50, 100); // Set bits 50-149
//! bitmap.clear_range(0, 50); // Clear bits 0-49
//!
//! // Check if a range is completely set or unset
//! assert!(bitmap.check_range_is_set(50, 100));
//! assert!(bitmap.check_range_is_unset(0, 50));
//!
//! // Iterate over a range
//! for (index, value) in bitmap.iter_range(40, 20) {
//!     println!("Bit {}: {}", index, value);
//! }
//!
//! // Count set bits
//! println!("Total set bits: {}", bitmap.count_ones());
//! println!("Total unset bits: {}", bitmap.count_zeros());
//! ```
//!
//! ### Flexible Bitmap (Dynamic size)
//!
//! ```rust
//! use ranged_bitmap::FlexBitMap;
//!
//! // Create a flexible bitmap that grows as needed
//! let mut bitmap = FlexBitMap::new();
//!
//! // Automatically grows when setting bits beyond current capacity
//! bitmap.set(1000); // Grows to accommodate bit 1000
//! bitmap.set_range(2000, 100); // Grows to accommodate the range
//!
//! // All operations work the same as fixed bitmap
//! assert!(bitmap.get(1000));
//! assert!(bitmap.check_range_is_set(2000, 100));
//!
//! // Check current capacity
//! println!("Current capacity: {} bits", bitmap.capacity());
//! println!("Number of blocks: {}", bitmap.blocks());
//! ```
//!
//! ### Small-Optimized Bitmap (Stack storage with heap growth)
//!
//! ```rust
//! use ranged_bitmap::generate_small_bit_map_struct;
//!
//! // Generate a SmallBitMap wrapper with 4 blocks on the stack (256 bits)
//! generate_small_bit_map_struct!(struct MySmallBitmap<4>);
//!
//! // Create a small-optimized bitmap
//! let mut bitmap = MySmallBitmap::new();
//!
//! // Small bitmaps (≤256 bits) use only stack storage
//! bitmap.set_range(0, 64);   // Stack-only, no heap allocation
//! bitmap.set_range(64, 128);  // Still stack-only
//!
//! // Large bitmaps automatically grow to heap when needed
//! bitmap.set_range(256, 1000); // Automatically spills to heap
//!
//! println!("Capacity: {} bits", bitmap.capacity());
//!
//! // All operations work the same as other bitmap types
//! assert!(bitmap.get(300));
//! assert!(bitmap.check_range_is_set(0, 192));
//!
//! // Perfect for cases where most bitmaps are small but occasionally grow
//! ```

#![deny(clippy::all)]
#![deny(clippy::assertions_on_result_states)]
#![deny(clippy::match_wild_err_arm)]
#![deny(clippy::allow_attributes_without_reason)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![allow(async_fn_in_trait, reason = "It improves readability.")]
#![allow(
    clippy::missing_const_for_fn,
    reason = "Since we cannot make a constant function non-constant after its release,
    we need to look for a reason to make it constant, and not vice versa."
)]
#![allow(clippy::inline_always, reason = "We write highly optimized code.")]
#![allow(
    clippy::must_use_candidate,
    reason = "It is better to developer think about it."
)]
#![allow(
    clippy::module_name_repetitions,
    reason = "This is acceptable most of the time."
)]
#![allow(
    clippy::missing_errors_doc,
    reason = "Unless the error is something special,
    the developer should document it."
)]
#![allow(clippy::redundant_pub_crate, reason = "It improves readability.")]
#![allow(clippy::struct_field_names, reason = "It improves readability.")]
#![allow(
    clippy::module_inception,
    reason = "It is fine if a file in has the same mane as a module."
)]
#![allow(clippy::if_not_else, reason = "It improves readability.")]
#![allow(
    rustdoc::private_intra_doc_links,
    reason = "It allows to create more readable docs."
)]

extern crate core;

#[cfg(test)]
extern crate alloc;

pub mod base;
pub(crate) mod fixed;
pub(crate) mod flex;
pub(crate) mod small;

pub use base::blocks_number_for_bits;
pub use fixed::FixedBitMap;
pub use flex::FlexBitMap;
pub use small::SmallBitMap;

/// Internal assertion helper for conditional debugging.
///
/// This function provides a way to include assertions that are only active
/// in debug builds or when the `more_checks` feature is enabled. This allows
/// for comprehensive bounds checking during development while maintaining
/// zero overhead in release builds.
#[inline(always)]
pub(crate) const fn maybe_assert(res: bool, msg: &'static str) {
    if cfg!(any(debug_assertions, feature = "more_checks")) {
        assert!(res, "{}", msg);
    } else {
        unsafe { core::hint::assert_unchecked(res) }
    }
}
