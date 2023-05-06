// leveldb/include/leveldb/slice.h

use std::alloc;
use std::cmp::Ordering;
use std::ffi::c_void;
use std::mem;
use std::ops::Index;
use std::os::raw::{c_char, c_int};

extern "C" {
    pub fn memcmp(p1: *const c_void, p2: *const c_void, n: usize) -> c_int;
}

unsafe fn memcmp_rs<T, U>(p1: *const T, p2: *const U, n: usize) -> i32 {
    memcmp(p1 as _, p2 as _, n) as i32
}

// TODO: Possibly impose lifetime using PhantomData
#[derive(Clone, Copy)]
pub struct Slice {
    data: *const c_char,
    size: usize,
}

impl Slice {
    pub fn new() -> Self {
        Default::default()
    }

    // NOTE: Possibly rename
    pub fn from_raw(data: *const c_char, size: usize) -> Self {
        Self { data, size }
    }

    pub fn data(&self) -> *const c_char {
        self.data
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn empty(&self) -> bool {
        self.size == 0
    }

    pub fn clear(&mut self) {
        self.data = "".as_ptr().cast();
        self.size = 0;
    }

    pub fn remove_prefix(&mut self, n: usize) {
        assert!(n <= self.size(), "remove index out of bounds");
        unsafe {
            self.data = self.data.add(n);
        }
        self.size -= n;
    }

    pub fn compare(&self, b: &Slice) -> i32 {
        let min_len = if self.size < b.size() {
            self.size
        } else {
            b.size()
        };
        let mut r = unsafe { memcmp_rs(self.data, b.data(), min_len) };
        if r == 0 {
            if self.size < b.size() {
                if self.size < b.size() {
                    r = -1
                } else if self.size > b.size() {
                    r = 1;
                }
            }
        }
        r
    }

    pub fn starts_with(&self, x: &Slice) -> bool {
        unsafe { self.size >= x.size() && memcmp_rs(self.data, x.data(), x.size()) == 0 }
    }
}

impl Default for Slice {
    fn default() -> Self {
        Self::from_raw("".as_ptr().cast(), 0)
    }
}

impl From<&str> for Slice {
    fn from(s: &str) -> Self {
        Self::from_raw(s.as_ptr().cast(), s.len())
    }
}

impl From<&String> for Slice {
    fn from(s: &String) -> Self {
        Self::from_raw(s.as_ptr().cast(), s.len())
    }
}

impl ToString for Slice {
    fn to_string(&self) -> String {
        unsafe {
            let l = self.size;
            // NOTE: Possibly use std::process::die
            let layout =
                alloc::Layout::from_size_align(l, mem::align_of::<u8>()).expect("bad layout");
            let buf = alloc::alloc(layout);
            buf.copy_from(self.data as *const u8, l);
            String::from_raw_parts(buf, l, l)
        }
    }
}

impl Index<usize> for Slice {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.size(), "index out of bounds");
        //unsafe { &(*self.data.add(index) as u8) }
        unsafe {
            self.data
                .add(index)
                .cast::<u8>()
                .as_ref()
                .expect("null pointer")
        }
    }
}

impl PartialEq for Slice {
    fn eq(&self, b: &Slice) -> bool {
        self.compare(b) == 0
    }
}

impl Eq for Slice {}

impl PartialOrd for Slice {
    fn partial_cmp(&self, b: &Slice) -> Option<Ordering> {
        match self.compare(b) {
            x if x < 0 => Some(Ordering::Less),
            0 => Some(Ordering::Equal),
            x if x > 0 => Some(Ordering::Greater),
            _ => unreachable!(),
        }
    }
}

impl Ord for Slice {
    fn cmp(&self, b: &Slice) -> Ordering {
        self.partial_cmp(b).unwrap()
    }
}
