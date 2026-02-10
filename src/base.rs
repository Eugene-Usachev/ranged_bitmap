use crate::maybe_assert;

const LAST: usize = usize::BITS as usize - 1;

#[inline(always)]
pub(crate) const fn blocks_number_for_bits(requested_bits: usize) -> usize {
    (requested_bits / usize::BITS as usize)
        + if requested_bits % usize::BITS as usize != 0 {
            1
        } else {
            0
        }
}

#[inline(always)]
pub(crate) const fn calc_block_and_slot(bit: usize) -> (usize, usize) {
    (bit / usize::BITS as usize, bit % usize::BITS as usize)
}

#[inline(always)]
pub(crate) const fn set(blocks: &mut [usize], bit: usize) {
    maybe_assert(
        bit < blocks.len() * usize::BITS as usize,
        "`bit` is out of bounds",
    );

    let (block, slot) = calc_block_and_slot(bit);

    blocks[block] |= 1 << slot;
}

#[inline(always)]
pub(crate) const fn clear(blocks: &mut [usize], bit: usize) {
    maybe_assert(
        bit < blocks.len() * usize::BITS as usize,
        "`bit` is out of bounds",
    );

    let (block, slot) = calc_block_and_slot(bit);

    blocks[block] &= !(1 << slot);
}

#[inline(always)]
pub(crate) const fn get(blocks: &[usize], bit: usize) -> bool {
    maybe_assert(
        bit < blocks.len() * usize::BITS as usize,
        "`bit` is out of bounds",
    );

    let (block, slot) = calc_block_and_slot(bit);

    blocks[block] & (1 << slot) != 0
}

#[inline(always)]
pub(crate) const fn xor(blocks: &mut [usize], bit: usize) {
    maybe_assert(
        bit < blocks.len() * usize::BITS as usize,
        "`bit` is out of bounds",
    );

    let (block, slot) = calc_block_and_slot(bit);

    blocks[block] ^= 1 << slot;
}

pub(crate) const fn count_ones(blocks: &[usize]) -> usize {
    let mut count = 0;
    let mut i = 0;

    while i < blocks.len() {
        count += blocks[i].count_ones() as usize;

        i += 1;
    }

    count
}

pub(crate) const fn count_zero(blocks: &[usize]) -> usize {
    let mut count = 0;
    let mut i = 0;

    while i < blocks.len() {
        count += blocks[i].count_zeros() as usize;

        i += 1;
    }

    count
}

pub(crate) const fn iter_range(
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
                let res = Some((self.curr, get(self.bitmap, self.curr)));

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

const fn update_range_more_than_two_blocks(
    blocks: &mut [usize],
    len: usize,
    set_bits_for_len: &[usize; usize::BITS as usize],
    start_bit: usize,
    start_block: usize,
    is_set_operation: bool,
) {
    let first_block_len = (LAST - start_bit) + 1;
    let first_mask = set_bits_for_len[first_block_len] << start_bit;

    if is_set_operation {
        blocks[start_block] |= first_mask;
    } else {
        blocks[start_block] &= first_mask;
    }

    let remaining = len - first_block_len;
    let mut remaining_full_blocks = remaining / usize::BITS as usize;
    let last_block_len = remaining % usize::BITS as usize;
    let last_mask = set_bits_for_len[last_block_len];

    if is_set_operation {
        blocks[start_block + remaining_full_blocks] |= last_mask;
    } else {
        blocks[start_block + remaining_full_blocks] &= last_mask;
    }

    let mut curr = start_block + 1;
    loop {
        if remaining_full_blocks < 2 {
            // remaining_full_blocks == 1
            if is_set_operation {
                blocks[curr] = set_bits_for_len[LAST];
            } else {
                blocks[curr] &= set_bits_for_len[LAST];
            }

            break;
        } else if remaining_full_blocks < 4 {
            if remaining_full_blocks == 2 {
                if is_set_operation {
                    blocks[curr] = set_bits_for_len[LAST];
                    blocks[curr + 1] = set_bits_for_len[LAST];
                } else {
                    blocks[curr] &= set_bits_for_len[LAST];
                    blocks[curr + 1] &= set_bits_for_len[LAST];
                }

                break;
            } else {
                if is_set_operation {
                    blocks[curr] = set_bits_for_len[LAST];
                    blocks[curr + 1] = set_bits_for_len[LAST];
                    blocks[curr + 2] = set_bits_for_len[LAST];
                } else {
                    blocks[curr] &= set_bits_for_len[LAST];
                    blocks[curr + 1] &= set_bits_for_len[LAST];
                    blocks[curr + 2] &= set_bits_for_len[LAST];
                }

                break;
            }
        } else if remaining_full_blocks < 8 {
            if is_set_operation {
                blocks[curr] = set_bits_for_len[LAST];
                blocks[curr + 1] = set_bits_for_len[LAST];
                blocks[curr + 2] = set_bits_for_len[LAST];
                blocks[curr + 3] = set_bits_for_len[LAST];
            } else {
                blocks[curr] &= set_bits_for_len[LAST];
                blocks[curr + 1] &= set_bits_for_len[LAST];
                blocks[curr + 2] &= set_bits_for_len[LAST];
                blocks[curr + 3] &= set_bits_for_len[LAST];
            }

            if remaining_full_blocks == 4 {
                break;
            }

            curr += 4;
            remaining_full_blocks -= 4;
        } else {
            if is_set_operation {
                blocks[curr] = set_bits_for_len[LAST];
                blocks[curr + 1] = set_bits_for_len[LAST];
                blocks[curr + 2] = set_bits_for_len[LAST];
                blocks[curr + 3] = set_bits_for_len[LAST];
                blocks[curr + 4] = set_bits_for_len[LAST];
                blocks[curr + 5] = set_bits_for_len[LAST];
                blocks[curr + 6] = set_bits_for_len[LAST];
                blocks[curr + 7] = set_bits_for_len[LAST];
            } else {
                blocks[curr] &= set_bits_for_len[LAST];
                blocks[curr + 1] &= set_bits_for_len[LAST];
                blocks[curr + 2] &= set_bits_for_len[LAST];
                blocks[curr + 3] &= set_bits_for_len[LAST];
                blocks[curr + 4] &= set_bits_for_len[LAST];
                blocks[curr + 5] &= set_bits_for_len[LAST];
                blocks[curr + 6] &= set_bits_for_len[LAST];
                blocks[curr + 7] &= set_bits_for_len[LAST];
            }

            if remaining_full_blocks == 8 {
                break;
            }

            curr += 8;
            remaining_full_blocks -= 8;
        }
    }
}

const fn update_range(
    blocks: &mut [usize],
    start: usize,
    len: usize,
    set_bits_for_len: &[usize; usize::BITS as usize],
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

    match () {
        () if len < usize::BITS as usize => {
            if start_bit + len < usize::BITS as usize {
                // one block
                let mask = set_bits_for_len[len] << start_bit;
                if is_set_operation {
                    blocks[block] |= mask;
                } else {
                    blocks[block] &= mask;
                }
            } else {
                // two blocks
                let first_block_len = (LAST - start_bit) + 1;
                let first_mask = set_bits_for_len[first_block_len] << start_bit;
                let second_mask = set_bits_for_len[len - first_block_len];

                if is_set_operation {
                    blocks[block] |= first_mask;
                    blocks[block + 1] |= second_mask;
                } else {
                    blocks[block] &= first_mask;
                    blocks[block + 1] &= second_mask;
                }
            }
        }
        () if len == usize::BITS as usize => {
            if start_bit == 0 {
                // exactly one full block
                if is_set_operation {
                    blocks[block] = set_bits_for_len[LAST];
                } else {
                    blocks[block] = 0;
                }
            } else {
                // two blocks
                let first_block_len = (LAST - start_bit) + 1;
                let first_mask = set_bits_for_len[first_block_len] << start_bit;
                let second_mask = set_bits_for_len[len - first_block_len];

                if is_set_operation {
                    blocks[block] |= first_mask;
                    blocks[block + 1] |= second_mask;
                } else {
                    blocks[block] &= first_mask;
                    blocks[block + 1] &= second_mask;
                }
            }
        }
        _ => {
            if start_bit + len < 2 * usize::BITS as usize {
                // two blocks
                let first_block_len = usize::BITS as usize - start_bit;
                let first_mask = set_bits_for_len[first_block_len] << start_bit;
                let second_mask = set_bits_for_len[len - first_block_len];

                if is_set_operation {
                    blocks[block] |= first_mask;
                    blocks[block + 1] |= second_mask;
                } else {
                    blocks[block] &= first_mask;
                    blocks[block + 1] &= second_mask;
                }
            } else {
                // more than two blocks
                update_range_more_than_two_blocks(blocks, len, set_bits_for_len, start_bit, block, is_set_operation);
            }
        }
    }
}

pub(crate) const fn set_range(blocks: &mut [usize], start: usize, len: usize) {
    const SET_BITS_FOR_LEN: [usize; usize::BITS as usize] = {
        let mut res = [0; usize::BITS as usize];

        let mut curr = 0;

        while curr < usize::BITS as usize {
            let mut i = 0;
            let mut block = 0;

            while i <= curr {
                block |= 1 << i;

                i += 1;
            }

            res[curr] = block;

            curr += 1;
        }

        res
    };

    update_range(blocks, start, len, &SET_BITS_FOR_LEN, true);
}

pub(crate) const fn clear_range(blocks: &mut [usize], start: usize, len: usize) {
    const RESET_BITS_FOR_LEN: [usize; usize::BITS as usize] = {
        let mut res = [0; usize::BITS as usize];

        let mut curr = 0;

        while curr < usize::BITS as usize {
            let mut i = 0;
            let mut block = !0; // Start with all bits set

            while i <= curr {
                block &= !(1 << i); // Clear the first curr bits
                i += 1;
            }

            res[curr] = block;

            curr += 1;
        }

        res
    };

    update_range(blocks, start, len, &RESET_BITS_FOR_LEN, false);
}

const fn check_range_more_than_two_blocks(
    blocks: &[usize],
    len: usize,
    check_bits_for_len: &[usize; usize::BITS as usize],
    start_bit: usize,
    start_block: usize,
    is_set_check: bool,
) -> bool {
    let first_block_len = usize::BITS as usize - start_bit;
    let first_block_mask = check_bits_for_len[first_block_len] << start_bit;

    let remaining = len - first_block_len;
    let remaining_full_blocks = remaining / usize::BITS as usize;
    let last_block_len = remaining % usize::BITS as usize;
    let last_block_mask = check_bits_for_len[last_block_len];

    let mut total_bits = 0;

    // Check first block
    let first_block_bits = (blocks[start_block] & first_block_mask).count_ones() as usize;
    total_bits += first_block_bits;

    let mut i = 1;

    // Check full blocks in the middle
    while i < remaining_full_blocks + 1 {
        let block_idx = start_block + i;
        if i == remaining_full_blocks && last_block_len == 0 {
            // This is actually the last block with full mask
            let full_mask = check_bits_for_len[LAST];
            let block_bits = (blocks[block_idx] & full_mask).count_ones() as usize;

            total_bits += block_bits;

            break;
        } else if i == remaining_full_blocks {
            // Last block with partial mask
            let block_bits = (blocks[block_idx] & last_block_mask).count_ones() as usize;

            total_bits += block_bits;

            break;
        } else {
            // Full block in the middle
            let full_mask = check_bits_for_len[LAST];
            let block_bits = (blocks[block_idx] & full_mask).count_ones() as usize;
            total_bits += block_bits;
        }

        i += 1;
    }

    // Check if total bits match expectation
    if is_set_check {
        total_bits == len
    } else {
        total_bits == 0
    }
}

const fn check_range_internal(
    blocks: &[usize],
    start: usize,
    len: usize,
    check_bits_for_len: &[usize; usize::BITS as usize],
    is_set_check: bool,
) -> bool {
    maybe_assert(
        start + len <= blocks.len() * usize::BITS as usize,
        "`start` + `len` is out of bounds",
    );

    let (block, start_bit) = calc_block_and_slot(start);

    match () {
        () if len < usize::BITS as usize => {
            if start_bit + len < usize::BITS as usize {
                let mask = check_bits_for_len[len] << start_bit;
                if is_set_check {
                    (blocks[block] & mask) == mask
                } else {
                    (blocks[block] & mask) == 0
                }
            } else {
                let first_block_len = (LAST - start_bit) + 1;
                let first_mask = check_bits_for_len[first_block_len] << start_bit;
                let second_mask = check_bits_for_len[len - first_block_len];

                if is_set_check {
                    (blocks[block] & first_mask) == first_mask
                        && (blocks[block + 1] & second_mask) == second_mask
                } else {
                    (blocks[block] & first_mask) == 0 && (blocks[block + 1] & second_mask) == 0
                }
            }
        }
        () if len == usize::BITS as usize => {
            if start_bit == 0 {
                // exactly one full block
                let mask = check_bits_for_len[LAST];
                if is_set_check {
                    (blocks[block] & mask) == mask
                } else {
                    (blocks[block] & mask) == 0
                }
            } else {
                // two blocks
                let first_block_len = usize::BITS as usize - start_bit;
                let first_mask = check_bits_for_len[first_block_len] << start_bit;
                let second_mask = check_bits_for_len[len - first_block_len];

                if is_set_check {
                    (blocks[block] & first_mask) == first_mask
                        && (blocks[block + 1] & second_mask) == second_mask
                } else {
                    (blocks[block] & first_mask) == 0 && (blocks[block + 1] & second_mask) == 0
                }
            }
        }
        _ => {
            if start_bit + len < 2 * usize::BITS as usize {
                let first_block_len = usize::BITS as usize - start_bit;
                let first_mask = check_bits_for_len[first_block_len] << start_bit;
                let second_mask = check_bits_for_len[len - first_block_len];

                if is_set_check {
                    (blocks[block] & first_mask) == first_mask
                        && (blocks[block + 1] & second_mask) == second_mask
                } else {
                    (blocks[block] & first_mask) == 0 && (blocks[block + 1] & second_mask) == 0
                }
            } else {
                check_range_more_than_two_blocks(
                    blocks,
                    len,
                    check_bits_for_len,
                    start_bit,
                    block,
                    is_set_check,
                )
            }
        }
    }
}

pub(crate) const fn check_range_is_set(blocks: &[usize], start: usize, len: usize) -> bool {
    const CHECK_BITS_FOR_LEN: [usize; usize::BITS as usize] = {
        let mut res = [0; usize::BITS as usize];

        let mut curr = 0;

        while curr < usize::BITS as usize {
            let mut i = 0;
            let mut block = 0;

            while i <= curr {
                block |= 1 << i;

                i += 1;
            }

            res[curr] = block;

            curr += 1;
        }

        res
    };

    check_range_internal(blocks, start, len, &CHECK_BITS_FOR_LEN, true)
}

pub(crate) const fn check_range_is_unset(blocks: &[usize], start: usize, len: usize) -> bool {
    const CHECK_BITS_FOR_LEN: [usize; usize::BITS as usize] = {
        let mut res = [0; usize::BITS as usize];

        let mut curr = 0;

        while curr < usize::BITS as usize {
            let mut i = 0;
            let mut block = 0;

            while i <= curr {
                block |= 1 << i;

                i += 1;
            }

            res[curr] = block;

            curr += 1;
        }

        res
    };

    check_range_internal(blocks, start, len, &CHECK_BITS_FOR_LEN, false)
}
