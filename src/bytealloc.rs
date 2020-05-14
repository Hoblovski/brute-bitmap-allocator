use crate::BitAllocator;

/// An unoptimized first-fit bitmap allocator.

const MAX_LEN: usize = 0x600000 / 4096;

pub struct LinearBitMap {
    size: usize,
    bitmap: [bool; MAX_LEN] // Allow concurrent access.
}

impl LinearBitMap {
    /// Allocate one bit. Fast-path.
    fn alloc_1(&mut self) -> Option<usize> {
        let bm = &mut self.bitmap;
        for i in 0..self.size {
            if !bm[i] {
                bm[i] = true;
                return Some(i);
            }
        }
        None
    }
}

impl BitAllocator for LinearBitMap {
    fn new(size: usize) -> Self {
        assert!(size <= MAX_LEN);
        LinearBitMap {
            size,
            bitmap: [false; MAX_LEN]
        }
    }

    fn alloc(&mut self, n: usize) -> Option<usize> {
        assert!(0 < n && n <= self.size);
        // fast path
        if n == 1 {
            return self.alloc_1();
        }
        // general case
        let bm = &mut self.bitmap;
        let mut begin = 0;
        while begin < self.size {
            while begin < self.size && bm[begin] { begin += 1; }
            if begin >= self.size { break; }
            let mut end = begin + 1;
            while end < self.size && !bm[end] { end += 1; }
            if end - begin >= n {
                for i in begin..begin+n { bm[i] = true; }
                return Some(begin);
            }
            begin = end;
        }
        None
    }

    fn alloc_aligned(&mut self, n: usize, align: usize) -> Option<usize> {
        assert!(0 < n && n <= self.size);
        assert!(align >= 1 && align & (align-1) == 0); // alignment must be a power of 2
        let align_mask = align - 1;
        let bm = &mut self.bitmap;
        let mut begin = 0;
        while begin < self.size {
            while begin < self.size && bm[begin] { begin += 1; }
            begin = (begin + align - 1) & !align_mask; // x >= begin s.t. x is aligned
            if begin >= self.size { break; }
            if bm[begin] { continue; } // advance to allocated position
            let mut end = begin + 1;
            while end < self.size && !bm[end] { end += 1; }
            if end - begin >= n {
                for i in begin..begin+n { bm[i] = true; }
                return Some(begin);
            }
            begin = end;
        }
        None
    }

    fn dealloc(&mut self, begin: usize, n: usize) {
        let bm = &mut self.bitmap;
        for i in begin..begin+n {
            bm[i] = false;
        }
    }
}
