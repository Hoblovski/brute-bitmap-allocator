#![no_std]

trait BitAllocator {
    /// Create a bitmap with size bits. By default, all bits are unallocated.
    fn new(size: usize) -> Self;
    /// Allocate n consecutive bits. Returns the index of the first.
    fn alloc(&mut self, n: usize) -> Option<usize>;
    /// Parameters:
    /// * align: must be a power of two.
    ///
    /// Similar to alloc, but the returned index is aligned to align.
    fn alloc_aligned(&mut self, n: usize, alignment: usize) -> Option<usize>;
    fn dealloc(&mut self, begin: usize, n: usize);
}

pub mod bitalloc;
pub mod bytealloc;

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{bitalloc as b};
    use crate::{bytealloc as B};

    fn inner_test_alloc<T: BitAllocator>(mut bm: T) {
        assert_eq!(Some(0), bm.alloc(5));
        assert_eq!(Some(5), bm.alloc(3));
        assert_eq!(None, bm.alloc(3));
        assert_eq!(Some(8), bm.alloc(2));
        assert_eq!(None, bm.alloc(1));
    }

    #[test]
    fn test_alloc() {
        inner_test_alloc(b::LinearBitMap::new(10));
        inner_test_alloc(B::LinearBitMap::new(10));
    }

    fn inner_test_dealloc<T: BitAllocator>(mut bm: T) {
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

    #[test]
    fn test_dealloc() {
        inner_test_dealloc(b::LinearBitMap::new(10));
        inner_test_dealloc(B::LinearBitMap::new(10));
    }

    fn inner_test_alloc_aligned<T: BitAllocator>(mut bm: T) {
        assert_eq!(Some(0), bm.alloc(5));
        assert_eq!(Some(6), bm.alloc_aligned(3, 2));
        assert_eq!(None, bm.alloc(2));
        assert_eq!(Some(5), bm.alloc(1));
        assert_eq!(Some(9), bm.alloc(1));
        assert_eq!(None, bm.alloc(1));
        bm.dealloc(4, 6);
        assert_eq!(Some(8), bm.alloc_aligned(2, 8));
        assert_eq!(Some(4), bm.alloc(1));
        assert_eq!(None, bm.alloc_aligned(2, 4));
        assert_eq!(Some(6), bm.alloc_aligned(2, 2));
        assert_eq!(Some(5), bm.alloc(1));
    }

    #[test]
    fn test_alloc_aligned() {
        inner_test_alloc_aligned(b::LinearBitMap::new(10));
        inner_test_alloc_aligned(B::LinearBitMap::new(10));
    }

    #[test]
    fn test_byte_bit_equivalent() {
        const N: usize = 1000;
        let mut bm_bit = crate::bitalloc::LinearBitMap::new(N);
        let mut bm_byte = crate::bytealloc::LinearBitMap::new(N);

        let randint = |b: usize, e: usize| { b + (rand::random::<usize>() % (e - b + 1)) };
        for i in 0..100000 {
            let opno: usize = randint(0, 2);
            match opno {
                0 => { // alloc
                    let n: usize = randint(1, N);
                    assert_eq!(bm_bit.alloc(n), bm_byte.alloc(n));
                }
                1 => { // dealloc
                    let b: usize = randint(0, N - 1);
                    let n: usize = randint(1, N - b);
                    assert_eq!(bm_bit.dealloc(b, n), bm_byte.dealloc(b, n));
                }
                _ => { // alloc_aligned
                    let n: usize = randint(1, N);
                    let a: usize = 1 << randint(1, 5);
                    assert_eq!(bm_bit.alloc_aligned(n, a), bm_byte.alloc_aligned(n, a));
                }
            }
        }
    }
}
