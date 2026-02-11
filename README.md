# Ranged Bitmap

A high-performance bitmap library.

## Features

- All functions are **constant**
- **Optimized range operations** that work on multiple bits simultaneously
- **No_std compatibility** for embedded and bare-metal environments

## Performance

This library is designed with performance as a primary goal:

- Range operations use precomputed lookup tables for optimal bit manipulation
- Bulk operations on full blocks use `memset`-like operations for maximum speed
- All functions are marked `#[inline(always)]` for aggressive inlining
- Constant functions enable compile-time optimizations
- Hardware-accelerated bit counting operations

## Basic Usage

```rust
use ranged_bitmap::generate_fixed_bit_map_struct;

generate_fixed_bit_map_struct!(struct BitMap<256>); // Generates a 256-bit bitmap struct wrapper

// Create a 256-bit bitmap
let mut bitmap = BitMap::new();

// Set individual bits
bitmap.set(10);
bitmap.set(20);

// Gets individual bits
assert!(bitmap.get(10));
assert!(!bitmap.get(21));

// Set an entire range at once (much faster than individual operations)
bitmap.set_range(50, 100); // Set bits 50-149
bitmap.clear_range(0, 50); // Clear bits 0-49

// Check if a range is completely set or unset
assert!(bitmap.check_range_is_set(50, 100));
assert!(bitmap.check_range_is_unset(0, 50));

// Iterate over a range
for (index, value) in bitmap.iter_range(40, 20) {
    println!("Bit {}: {}", index, value);
}

// Count set bits
println!("Total set bits: {}", bitmap.count_ones());
println!("Total unset bits: {}", bitmap.count_zeros());
```

### Range Operations

The library excels at a range of operations:

```rust
use ranged_bitmap::FixedBitMap;

let mut bitmap = FixedBitMap::<2>::new(); // 128 bits

// Efficiently set large ranges
bitmap.set_range(0, 64);   // Set first block
bitmap.set_range(64, 64);  // Set second block

// Check range status
assert!(bitmap.check_range_is_set(0, 128));

// Clear ranges efficiently
bitmap.clear_range(32, 64); // Clear middle portion
```

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Individual bit get/set/clear | O(1) | Single memory access |
| Range operations (single block) | O(1) | Bitmask operations |
| Range operations (multi-block) | O(n) | n = number of affected blocks |
| Full block operations | O(n) | memset-like optimizations |
| Bit counting | O(n) | Hardware-accelerated |

## License

This project is licensed under the MIT License.