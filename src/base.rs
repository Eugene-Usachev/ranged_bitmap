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
    set_bits_for_len: &[usize; usize::BITS as usize + 1],
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

const fn get_ref_by_index<T>(arr: &[T], idx: usize) -> &T {
    if cfg!(test) {
        &arr[idx]
    } else {
        unsafe { &*arr.as_ptr().add(idx) }
    }
}

const fn get_mut_by_index<T>(arr: &mut [T], idx: usize) -> &mut T {
    if cfg!(test) {
        &mut arr[idx]
    } else {
        unsafe { &mut *arr.as_mut_ptr().add(idx) }
    }
}

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
            *get_mut_by_index(blocks, block) &= mask;
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
            *get_mut_by_index(blocks, block) &= first_mask;
            *get_mut_by_index(blocks, block + 1) &= second_mask;
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
            *get_mut_by_index(blocks, block) &= first_mask;

            if last_block_len > 0 {
                let second_mask = *get_ref_by_index(set_bits_for_len, last_block_len);

                *get_mut_by_index(blocks, block + number_of_full_blocks + 1) &= second_mask;
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

pub(crate) const fn set_range(blocks: &mut [usize], start: usize, len: usize) {
    update_range(blocks, start, len, &SET_BITS_FOR_LEN, true);
}

const RESET_BITS_FOR_LEN: [usize; usize::BITS as usize + 1] = {
    let mut res = [0; usize::BITS as usize + 1];

    let mut curr = 0;

    while curr < usize::BITS as usize {
        let mut i = 0;
        let mut block = !0; // Start with all bits set

        while i <= curr {
            block &= !(1 << i); // Clear the first curr bits
            i += 1;
        }

        res[curr + 1] = block;

        curr += 1;
    }

    res
};

pub(crate) const fn clear_range(blocks: &mut [usize], start: usize, len: usize) {
    update_range(blocks, start, len, &RESET_BITS_FOR_LEN, false);
}

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

pub(crate) const fn check_range_is_set(blocks: &[usize], start: usize, len: usize) -> bool {
    check_range_internal(blocks, start, len, &SET_BITS_FOR_LEN, true)
}

pub(crate) const fn check_range_is_unset(blocks: &[usize], start: usize, len: usize) -> bool {
    check_range_internal(blocks, start, len, &SET_BITS_FOR_LEN, false)
}
