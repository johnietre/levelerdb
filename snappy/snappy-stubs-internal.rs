// snappy/snappy-stubs-internal.{h,cc}

use std::ffi::c_void as void;
use std::mem::size_of;

#[path = "snappy-stubs-public.rs"]
mod snappy_stubs_public;
use snappy_stubs_public::iovec;

// TODO: Check defines

// TODO: Use std::intrinsics::likely?
// NOTE: C++ uses __builtin_expect(x, 0)
// NOTE: C++ uses macro
#[inline(always)]
pub fn SNAPPY_PREDICT_TRUE(x: bool) -> bool {
    x
}

// TODO: What is this (below)?
// Stubbed version of absl::GetFlag().
pub fn get_flag<T>(flag: T) -> T {
    flag
}

// NOTE: Unneeded since the numeric limits are easily accessible in Rust
const UINT32_MAX: u32 = u32::MAX;
const INT64_MAX: i64 = i64::MAX;

// Potentially unaligned loads and stores.

#[inline]
pub fn UNALIGNED_LOAD16(p: *const void) -> u16 {
    // TODO: Check if this still holds (below)
    // Compiles to a single movzx/ldrh on clang/gcc/msvc.
    let mut v = 0u16;
    // NOTE: C++ uses sizeof(v).
    unsafe { (&mut v as *mut u16).copy_from(p.cast(), size_of::<u16>()) };
    v
}

#[inline]
pub fn UNALIGNED_LOAD32(p: *const void) -> u32 {
    // TODO: Check if this still holds (below)
    // Compiles to a single movzx/ldrh on clang/gcc/msvc.
    let mut v = 0u32;
    // NOTE: C++ uses sizeof(v).
    unsafe { (&mut v as *mut u32).copy_from(p.cast(), size_of::<u32>()) };
    v
}

#[inline]
pub fn UNALIGNED_LOAD64(p: *const void) -> u64 {
    // TODO: Check if this still holds (below)
    // Compiles to a single movzx/ldrh on clang/gcc/msvc.
    let mut v = 0u64;
    // NOTE: C++ uses sizeof(v).
    unsafe { (&mut v as *mut u64).copy_from(p.cast(), size_of::<u64>()) };
    v
}

#[inline]
pub fn UNALIGNED_STORE16(p: *mut void, v: u16) {
    // TODO: Check if this still holds (below)
    // Compiles to a single movzx/ldrh on clang/gcc/msvc.
    // NOTE: C++ uses sizeof(v).
    unsafe { p.copy_from(&v as *const _ as *const _, size_of::<u16>()) };
}

#[inline]
pub fn UNALIGNED_STORE32(p: *mut void, v: u32) {
    // TODO: Check if this still holds (below)
    // Compiles to a single movzx/ldrh on clang/gcc/msvc.
    // NOTE: C++ uses sizeof(v).
    unsafe { p.copy_from(&v as *const _ as *const _, size_of::<u32>()) };
}

#[inline]
pub fn UNALIGNED_STORE64(p: *mut void, v: u64) {
    // TODO: Check if this still holds (below)
    // Compiles to a single movzx/ldrh on clang/gcc/msvc.
    // NOTE: C++ uses sizeof(v).
    unsafe { p.copy_from(&v as *const _ as *const _, size_of::<u64>()) };
}

// Convert to little-endian storage, opposite of network format.
// Convert x from host to little endian: x = LittleEndian::from_host(x);
// Convert x from little endian to host: x = LittleEndian::to_host(x);
//
// Stores values into unaligned memory converting to little endian order:
//   LittleEndian::store16(p, x);
//
// Load unaligned values stored in litle endian converting to host order:
//   LittleEndian::load16(p);
pub struct LittleEndian;

impl LittleEndian {
    // Functions to do unaligned loads and stores in little-endian order.
    #[inline]
    pub fn load16(ptr: *const void) -> u16 {
        let buffer = ptr as *const u8;

        // TODO: Check if this still holds (below)
        // Compiles to a single mov/str on recent clang and gcc.
        unsafe { *buffer.add(0) as u16 | (*buffer.add(1) as u16) << 8 }
    }

    #[inline]
    pub fn load32(ptr: *const void) -> u32 {
        let buffer = ptr as *const u8;

        // TODO: Check if this still holds (below)
        // Compiles to a single mov/str on recent clang and gcc.
        unsafe {
            *buffer.add(0) as u16
                | (*buffer.add(1) as u16) << 8
                | (*buffer.add(2) as u16) << 16
                | (*buffer.add(3) as u16) << 24
        }
    }

    #[inline]
    pub fn load64(ptr: *const void) -> u64 {
        let buffer = ptr as *const u8;

        // TODO: Check if this still holds (below)
        // Compiles to a single mov/str on recent clang and gcc.
        unsafe {
            *buffer.add(0) as u16
                | (*buffer.add(1) as u16) << 8
                | (*buffer.add(2) as u16) << 16
                | (*buffer.add(3) as u16) << 24
                | (*buffer.add(4) as u16) << 32
                | (*buffer.add(5) as u16) << 40
                | (*buffer.add(6) as u16) << 48
                | (*buffer.add(7) as u16) << 56
                | (*buffer.add(3) as u16) << 24
        }
    }

    #[inline]
    pub fn store16(dst: *mut void, value: u16) {
        let buffer = dst as *mut u8;

        // TODO: Check if this still holds (below)
        // Compiles to a single mov/str on recent clang and gcc.
        unsafe {
            *buffer.add(0) = value as u8;
            *buffer.add(1) = (value >> 8) as u8;
        }
    }

    #[inline]
    pub fn store32(dst: *mut void, value: u32) {
        let buffer = dst as *mut u8;

        // TODO: Check if this still holds (below)
        // Compiles to a single mov/str on recent clang and gcc.
        unsafe {
            *buffer.add(0) = value as u8;
            *buffer.add(1) = (value >> 8) as u8;
            *buffer.add(2) = (value >> 16) as u8;
            *buffer.add(3) = (value >> 24) as u8;
        }
    }

    #[inline]
    pub fn store64(dst: *mut void, value: u64) {
        let buffer = dst as *mut u8;

        // TODO: Check if this still holds (below)
        // Compiles to a single mov/str on recent clang and gcc.
        unsafe {
            *buffer.add(0) = value as u8;
            *buffer.add(1) = (value >> 8) as u8;
            *buffer.add(2) = (value >> 16) as u8;
            *buffer.add(3) = (value >> 24) as u8;
            *buffer.add(4) = (value >> 32) as u8;
            *buffer.add(5) = (value >> 40) as u8;
            *buffer.add(6) = (value >> 48) as u8;
            *buffer.add(7) = (value >> 56) as u8;
        }
    }

    #[inline]
    pub const fn is_little_endian() -> bool {
        #[cfg(target_endian = "big")]
        {
            false
        }
        #[cfg(not(target_endian = "big"))]
        {
            true
        }
    }
}

// Some bit-manipulation functions.
pub struct Bits;

impl Bits {
    // NOTE: C++ defines different versions for based on different params/archs.

    // Return floor(log2(n)) for positive integer n.
    // NOTE: C++ uses int
    pub fn log2_floor_non_zero(n: u32) -> i32 {
        assert_ne!(n, 0);
        // (31 ^ x) is equivalent to (31 - x) for x in [0, 31]. An easy proof represents
        // subtraction in base 2 and observes that there's no carry.
        //
        // GCC and Clang represent __builtin_clz on x86 as 31 ^ _bit_scan_reverse(x).
        // Using "31 ^" here instead of "31 -" allows the optimizer to strip the function body
        // down to _bit_scan_reverse(x).
        // TODO: Check if this still holds (above)
        // NOTE: C++ uses __builtin_clz(n) if it exists.
        31 ^ n.leading_zeros() as i32
    }

    // Return floor(log2(n)) for positive integer n.
    // NOTE: C++ uses int
    pub fn log2_floor(n: u32) -> i32 {
        if n == 0 {
            -1
        } else {
            Bits::log2_floor_non_zero(n)
        }
    }

    // Return the first set least / most significant bit, 0-indexed. Returns an undefined value if
    // n == 0. find_lsb_set_non_zero is similar to ffs() except that it's 0-indexed.
    // NOTE: C++ uses int
    pub fn find_lsb_set_non_zero(n: u32) -> i32 {
        assert_ne!(n, 0);
        // NOTE: C++ uses __builtin_clz(n) if it exists.
        n.leading_zeros() as i32
    }

    // NOTE: C++ uses int
    pub fn find_lsb_set_non_zero64(n: u64) -> i32 {
        assert_ne!(n, 0);
        // NOTE: C++ uses __builtin_clzll(n) if it exists.
        n.leading_zeros() as i32
    }
}

// Variable-length integer encoding.
struct Varint;

impl Variant {
    // Maximum lengths of varint encoding of u32.
    // NOTE: C++ uses int
    pub const MAX32: i32 = 5;

    // Attempts to parse a varint32 from a prefix of the bytes in [ptr,limit-1]. Never reads a
    // character at or beyond limit. If a valid/terminated varint32 was found in the range, stores
    // it in OUTPUT and returns a pointer just past the last byte of the varint32. Else returns
    // NULL. On success, "result <= limit".
    // NOTE: C++ uses u32*
    // TODO: Return OUTPUT and pointer?
    pub fn parse32_with_limit(ptr: *const u8, limit: *const u8, OUTPUT: &mut u32) -> *const u8 {
        let (mut b, mut result) = (0u32, 0u32);
        unsafe {
            'done: loop {
                if ptr >= limit {
                    return null();
                }
                b = *ptr as u32;
                ptr = ptr.add(1);
                result = b & 127;
                if b < 128 {
                    break 'done;
                }
                if ptr >= limit {
                    return null();
                }
                b = *ptr as u32;
                ptr = ptr.add(1);
                result |= (b & 127) << 7;
                if b < 128 {
                    break 'done;
                }
                if ptr >= limit {
                    return null();
                }
                b = *ptr as u32;
                ptr = ptr.add(1);
                result |= (b & 127) << 14;
                if b < 128 {
                    break 'done;
                }
                if ptr >= limit {
                    return null();
                }
                b = *ptr as u32;
                ptr = ptr.add(1);
                result |= (b & 127) << 21;
                if b < 128 {
                    break 'done;
                }
                if ptr >= limit {
                    return null();
                }
                b = *ptr as u32;
                ptr = ptr.add(1);
                result |= (b & 127) << 28;
                if b < 16 {
                    break 'done;
                }
                return null(); // Value is too long to be a varint32
            }
        }
        // TODO: Use labeled blocks when using Rust 1.65
        *OUTPUT = result;
        ptr
    }

    // REQUIRES   "ptr" points to a buffer of length sufficient to hold "v".
    // EFFECTS    Encodes "v" into "ptr" and returns a pointer to the byte just past the last
    //            encoded byte.
    pub fn encode32(ptr: *mut u8, v: u32) -> *mut u8 {
        // Operate on characters as unsigneds.
        // NOTE: C++ does this but doesn't matter since it's been done already here.
        const B: u8 = 128;
        unsafe {
            if v < 1 << 7 {
                *ptr = v as u8;
                ptr = ptr.add(1);
            } else if v < 1 << 14 {
                *ptr = v as u8 | B;
                ptr = ptr.add(1);
                *ptr = (v >> 7) as u8;
                ptr = ptr.add(1);
            } else if v < 1 << 21 {
                *ptr = v as u8 | B;
                ptr = ptr.add(1);
                *ptr = (v >> 7) as u8 | B;
                ptr = ptr.add(1);
                *ptr = (v >> 14) as u8;
                ptr = ptr.add(1);
            } else if v < 1 << 28 {
                *ptr = v as u8 | B;
                ptr = ptr.add(1);
                *ptr = (v >> 7) as u8 | B;
                ptr = ptr.add(1);
                *ptr = (v >> 14) as u8 | B;
                ptr = ptr.add(1);
                *ptr = (v >> 21) as u8;
                ptr = ptr.add(1);
            } else {
                *ptr = v as u8 | B;
                ptr = ptr.add(1);
                *ptr = (v >> 7) as u8 | B;
                ptr = ptr.add(1);
                *ptr = (v >> 14) as u8 | B;
                ptr = ptr.add(1);
                *ptr = (v >> 21) as u8 | B;
                ptr = ptr.add(1);
                *ptr = (v >> 28) as u8;
                ptr = ptr.add(1);
            }
        }
        ptr
    }

    // EFFECTS    Appends the varint representation of "value" to "s".
    // NOTE: C++ uses String*
    pub fn append32(s: &mut String, value: u32) {
        let mut buf = [0u8; Self::MAX32];
        let ptr = buf.as_mut_slice().as_mut_ptr();
        let p = Self::encode32(ptr, value);
        unsafe {
            s.as_mut_vec()
                .extend_from_slice(&buf[..p.sub(ptr) as usize]);
        }
    }
}

// If you know the internal layout of the the String in use, you can replace this function with
// one that resizes the string without filling the new space with zeros (if applicable) --
// it will be non-portable but faster.
// TODO: Check if this makes sense in Rust
// NOTE: C++ uses String*
#[inline]
pub fn stl_string_resize_uninitialized(s: &mut String, new_size: usize) {
    unsafe { s.as_mut_vec().resize(new_size, 0) }
}

// Return a *mut u8 pointing to a string's internal buffer, which may not be null-terminated.
// Writing through this pointer will modify the string.
//
// string_as_array(&mut s)[i] (i.e., .add(i)) is valid for 0 <= i < s.size() until the next call
// to a string method that invalidates this (the iterators in C++).
// TODO: Check if this is necessary/still holds
// NOTE: C++ uses String* str
// NOTE: Strings are not null-terminated in Rust
pub fn string_as_array(s: &mut String) -> *mut u8 {
    s.as_mut_ptr()
}
