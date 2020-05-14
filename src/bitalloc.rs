use core::ops::{Index, IndexMut, Add, AddAssign, Sub};
use crate::BitAllocator;

/// An unoptimized first-fit bitmap allocator.

const ELEM_WIDTH: usize = 64;
const ELEM_CNT: usize = 0x600000 / 4096 / ELEM_WIDTH;

#[derive(PartialOrd, PartialEq, Debug, Copy, Clone)]
struct RawIndex(usize, usize); // index, bit

impl RawIndex {
    fn new() -> Self { RawIndex(0, 0) }

    fn max() -> Self { RawIndex(ELEM_CNT - 1, ELEM_WIDTH - 1) }

    fn to_int(&self) -> usize { self.0 * ELEM_WIDTH + self.1 }

    fn from_int(x: usize) -> Self { RawIndex(x / ELEM_WIDTH, x % ELEM_WIDTH) }

    fn next_aligned(&self, alignment: usize) -> Self {
        let mask = alignment - 1;
        Self::from_int((self.to_int() + alignment - 1) & !mask)
    }
}

impl Add<usize> for RawIndex {
    type Output = RawIndex;

    fn add(self, rhs: usize) -> Self::Output {
        let (mut a, mut b) = (self.0, self.1 + rhs);
        a += (b / ELEM_WIDTH);
        b %= ELEM_WIDTH;
        RawIndex(a, b)
    }
}

impl AddAssign<usize> for RawIndex {
    fn add_assign(&mut self, rhs: usize) {
        self.1 += rhs;
        self.0 += self.1 / ELEM_WIDTH;
        self.1 %= ELEM_WIDTH;
    }
}

impl Sub for RawIndex {
    type Output = usize;

    fn sub(self, rhs: Self) -> Self::Output {
        self.to_int() - rhs.to_int()
    }
}

struct RawBitMap([i64; ELEM_CNT]);

impl RawBitMap {
    fn get(&self, i: &RawIndex) -> bool {
        assert!(i.0 <= ELEM_CNT);
        assert!(i.1 < 64);
        (self.0[i.0] >> i.1) & 1 == 1
    }

    fn set(&mut self, i: &RawIndex, b: bool) {
        assert!(i.0 <= ELEM_CNT);
        assert!(i.1 < 64);
        self.0[i.0] &= !(1 << i.1);
        self.0[i.0] |= (if b {1} else {0} << i.1);
    }

    fn set_range(&mut self, begin: &RawIndex, end: &RawIndex, b: bool) {
        let mut i = *begin;
        while i < *end {
            self.set(&i, b);
            i += 1;
        }
    }
}

pub struct LinearBitMap {
    size: usize,
    bitmap: RawBitMap,
    end: RawIndex
}

fn div_ceil(a: usize, b: usize) -> usize { (a+b-1)/b }

impl LinearBitMap {
    /// Allocate one bit. Fast-path.
    fn alloc_1(&mut self) -> Option<usize> {
        let bm = &mut self.bitmap;
        let mut i = RawIndex::new();
        while i < self.end {
            if !bm.get(&i) {
                bm.set(&i, true);
                return Some(i.to_int());
            }
            i += 1;
        }
        None
    }

    // If some, result > begin and bm[result] != bm[begin]
    // Could return self.end
    //
    // TODO: to speed up, multiple bits can be skipped at once
    fn next_toggle(&mut self, begin: &RawIndex) -> RawIndex {
        let b = self.bitmap.get(begin);
        let mut i = *begin;
        while i < self.end && self.bitmap.get(&i) == b {
            i += 1;
        }
        return i;
    }

    // TODO: same optimization as above
    fn first_of(&mut self, b: bool) -> RawIndex {
        let i = RawIndex::new();
        if self.bitmap.get(&i) == b { return i; }
        self.next_toggle(&i)
    }
}

impl BitAllocator for LinearBitMap {
    fn new(size: usize) -> Self {
        assert!(size <= ELEM_CNT * ELEM_WIDTH);
        LinearBitMap {
            size: size,
            bitmap: RawBitMap([0; ELEM_CNT]),
            end: RawIndex(size / ELEM_WIDTH, size % ELEM_WIDTH),
        }
    }

    fn alloc(&mut self, n: usize) -> Option<usize> {
        assert!(0 < n && n <= self.size * ELEM_WIDTH);
        // fast path
        if n == 1 {
            return self.alloc_1();
        }
        // general case
        let mut begin = self.first_of(false);
        loop {
            if begin == self.end { return None; }
            let end = self.next_toggle(&begin);
            if end - begin >= n {
                self.bitmap.set_range(&begin, &(begin + n), true);
                return Some(begin.to_int());
            }
            begin = self.next_toggle(&end);
        }
    }

    fn alloc_aligned(&mut self, n: usize, alignment: usize) -> Option<usize> {
        assert!(0 < n && n <= self.size);
        assert!(alignment >= 1 && alignment & (alignment -1) == 0); // alignment must be a power of 2
        let mut begin = self.first_of(false);
        loop {
            if begin == self.end { return None; }
            let end = self.next_toggle(&begin);
            begin = begin.next_aligned(alignment);
            if end > begin && end - begin >= n {
                self.bitmap.set_range(&begin, &(begin + n), true);
                return Some(begin.to_int());
            }
            begin = self.next_toggle(&end);
        }
    }

    fn dealloc(&mut self, begin: usize, n: usize) {
        self.bitmap.set_range(&RawIndex::from_int(begin),
                              &RawIndex::from_int(begin + n), false);
    }
}
