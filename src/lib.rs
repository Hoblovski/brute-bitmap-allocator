#![no_std]

/// An unoptimized first-fit bitmap allocator.

use spin::Mutex;

const MAX_LEN: usize = 0x600000 / 4096;

pub struct LinearBitMap {
    size: usize,
    bitmap: Mutex<[bool; MAX_LEN]> // Allow concurrent access.
}

impl LinearBitMap {
    /// Create a bitmap with size bits. By default, all bits are unallocated.
    pub fn new(size: usize) -> Self {
        assert!(size <= MAX_LEN);
        LinearBitMap {
            size,
            bitmap: Mutex::new([false; MAX_LEN])
        }
    }

    /// Allocate one bit. Fast-path.
    fn alloc_1(&self) -> Option<usize> {
        let mut bm = self.bitmap.lock();
        for i in 0..self.size {
            if !bm[i] {
                bm[i] = true;
                return Some(i);
            }
        }
        None
    }

    /// Allocate n consecutive bits. Returns the index of the first.
    pub fn alloc(&self, n: usize) -> Option<usize> {
        assert!(0 < n && n <= self.size);
        // fast path
        if n == 1 {
            return self.alloc_1();
        }
        // general case
        let mut bm = self.bitmap.lock();
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

    pub fn dealloc(&self, begin: usize, n: usize) {
        let mut bm = self.bitmap.lock();
        for i in begin..begin+n {
            assert!(bm[i]); // We cannot deallocate free bits.
            bm[i] = false;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc() {
        let bm = LinearBitMap::new(10);

        assert_eq!(Some(0), bm.alloc(5));
        assert_eq!(Some(5), bm.alloc(3));
        assert_eq!(None, bm.alloc(3));
        assert_eq!(Some(8), bm.alloc(2));
        assert_eq!(None, bm.alloc(1));
    }

    #[test]
    fn test_dealloc() {
        let bm = LinearBitMap::new(10);

        assert_eq!(Some(0), bm.alloc(5));
        assert_eq!(Some(5), bm.alloc(3));
        assert_eq!(None, bm.alloc(3));
        assert_eq!(Some(8), bm.alloc(2));
        assert_eq!(None, bm.alloc(1));
        bm.dealloc(4, 5);
        assert_eq!(Some(4), bm.alloc(1));
        assert_eq!(Some(5), bm.alloc(1));
        assert_eq!(Some(6), bm.alloc(1));
        assert_eq!(None, bm.alloc(3));
        assert_eq!(Some(7), bm.alloc(2));
    }
}
