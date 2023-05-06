// leveldb/util/arena.{h,cc}
// leveldb/util/arena_test.cc

use std::alloc;
use std::ffi::c_void;
use std::mem::size_of;
use std::os::raw::c_char;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

macro_rules! static_assert {
    ($x:expr) => {
        const _: usize = $x as bool as usize - 1;
    };
}

const BLOCK_SIZE: usize = 4096;

pub struct Arena {
    // Allocation state
    alloc_ptr: *mut c_char,
    alloc_bytes_remaining: usize,

    // Array of `alloc`ed memory blocks
    // NOTE: Use "fat" pointer
    blocks: Vec<(*mut c_char, usize)>,

    // Total memory usage of the arena.
    //
    // TODO(costan): This member is accessed via atomics, but the others are accessed without any
    // locking. Is this OK?
    memory_usage: Arc<AtomicUsize>,
}

impl Arena {
    pub fn new() -> Self {
        Self {
            // Can use null "safely" since it won't ever be dereferenced (I believe)
            alloc_ptr: std::ptr::null_mut(),
            alloc_bytes_remaining: 0,
            blocks: Vec::new(),
            memory_usage: Arc::new(AtomicUsize::new(0)),
        }
    }

    // Return a pointer to a newly allocated memory block of "bytes" bytes.
    #[inline]
    pub fn allocate(&mut self, bytes: usize) -> *mut c_char {
        // The semantics of what to return are a bit messy if we allow 0-byte allocations, so we
        // disallow them here (we don't need them for our internal use).
        assert!(bytes > 0);
        if bytes <= self.alloc_bytes_remaining {
            let res = self.alloc_ptr;
            self.alloc_ptr = unsafe { self.alloc_ptr.add(bytes) };
            self.alloc_bytes_remaining -= bytes;
            return res;
        }
        self.allocate_fallback(bytes)
    }

    // Allocate memory with the normal alignment guarantees provided by malloc.
    pub fn allocate_aligned(&mut self, bytes: usize) -> *mut c_char {
        const ALIGN: usize = if size_of::<*const c_void>() > 8 {
            size_of::<*const c_void>()
        } else {
            8
        };
        // Pointer size should be a power of 2
        static_assert!(ALIGN & (ALIGN - 1) == 0);
        // NOTE: C++ uses uintptr_t rather than size_t
        let current_mod = self.alloc_ptr as usize & (ALIGN - 1);
        let slop = if current_mod == 0 {
            0
        } else {
            ALIGN - current_mod
        };
        let needed = bytes + slop;
        let res;
        if needed <= self.alloc_bytes_remaining {
            res = unsafe { self.alloc_ptr.add(slop) };
            self.alloc_ptr = unsafe { self.alloc_ptr.add(needed) };
            self.alloc_bytes_remaining -= needed;
        } else {
            // AllocateFallback always returned aligned memory
            res = self.allocate_fallback(bytes);
        }
        assert!(res as usize & (ALIGN - 1) == 0);
        res
    }

    // Returns an estimate of the total memory usage of data allocated by the arena.
    pub fn memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }

    fn allocate_fallback(&mut self, bytes: usize) -> *mut c_char {
        if bytes > BLOCK_SIZE / 4 {
            // Object is more than a quarter of our block size. Allocate it separately to avoid
            // wasting too much space in leftover bytes.
            return self.allocate_new_block(bytes);
        }
        // We waste the remaining space in the current block.
        self.alloc_ptr = self.allocate_new_block(BLOCK_SIZE);
        self.alloc_bytes_remaining = BLOCK_SIZE;

        let res = self.alloc_ptr;
        self.alloc_ptr = unsafe { self.alloc_ptr.add(bytes) };
        self.alloc_bytes_remaining -= bytes;
        res
    }

    fn allocate_new_block(&mut self, block_bytes: usize) -> *mut c_char {
        let res = unsafe {
            alloc::alloc(alloc::Layout::array::<c_char>(block_bytes).unwrap()) as *mut c_char
        };
        self.blocks.push((res, block_bytes));
        self.memory_usage
            .fetch_add(block_bytes + size_of::<c_char>(), Ordering::Relaxed);
        res
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        for &(p, s) in self.blocks.iter() {
            unsafe {
                alloc::dealloc(p as *mut u8, alloc::Layout::array::<c_char>(s).unwrap());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::random::Random;

    #[test]
    fn test_empty() {
        let _ = Arena::new();
    }

    #[test]
    fn test_simple() {
        let mut allocated = Vec::new();
        let mut arena = Arena::new();
        const N: usize = 100_000;
        let mut bytes = 0usize;
        let mut rnd = Random::new(301);
        for i in 0..N {
            let mut s = if i % (N / 10) == 0 {
                i
            } else if rnd.one_in(4000) {
                rnd.uniform(6000) as usize
            } else if rnd.one_in(10) {
                rnd.uniform(100) as usize
            } else {
                rnd.uniform(20) as usize
            };
            if s == 0 {
                // Our arena disallows size 0 allocations.
                s = 1;
            }
            let r = if rnd.one_in(10) {
                arena.allocate_aligned(s)
            } else {
                arena.allocate(s)
            };

            for b in 0..s as usize {
                // Fill the "i"th allocation with a known bit pattern
                unsafe {
                    *r.add(b) = (i % 256) as c_char;
                }
            }
            bytes += s;
            allocated.push((s, r));
            assert!(arena.memory_usage() > bytes);
            if i > N / 10 {
                assert!(arena.memory_usage() < (bytes as f64 * 1.1) as usize);
            }
        }
        for i in 0..allocated.len() {
            let (num_bytes, p) = allocated[i];
            for b in 0..num_bytes {
                // Check the "i"th allocation for the known bit pattern
                unsafe {
                    // NOTE: C++ uses int
                    assert_eq!(*p.add(b) as i32 & 0xff, i as i32 % 256);
                }
            }
        }
    }
}
