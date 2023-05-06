// leveldb/util/coding.{h,cc}
// leveldb/util/coding_test.cc

#![allow(dead_code)]
use crate::slice::Slice;
use std::mem::size_of;
use std::os::raw::c_char;
use std::ptr::null;

// Standard Put.. routines append to a string
fn put_fixed32(dst: &mut String, value: u32) {
    // NOTE: Possibly use array and get ptr via arr.as_mut_slice().as_mut_ptr()
    // NOTE: C++ uses sizeof(value)
    // NOTE: Switch to 'size_of_val' when it becomes const
    const VAL_SIZE: usize = size_of::<u32>();
    let buf = &mut [0 as c_char; VAL_SIZE];
    encode_fixed32(buf.as_mut_ptr(), value);
    // NOTE: C++ appends sizeof(buf) of buf to string
    // Due to the way Rust stores the bytes of strings (e.g., a byte over 127 is constitutes 2
    // bytes of length), therefore, in order to get the desired layout, append a string of null
    // characters (or any character under 128), and overwrite the memory of the appended string
    // with the data from 'buf'
    let l = dst.len();
    dst.push_str(&"\0".repeat(VAL_SIZE));
    unsafe {
        dst.as_mut_ptr()
            .add(l)
            .copy_from(buf.as_ptr().cast(), VAL_SIZE);
    }
}

fn put_fixed64(dst: &mut String, value: u64) {
    // NOTE: Possibly use array and get ptr via arr.as_mut_slice().as_mut_ptr()
    // NOTE: C++ uses sizeof(value)
    const VAL_SIZE: usize = size_of::<u64>();
    let buf = &mut [0 as c_char; VAL_SIZE];
    encode_fixed64(buf.as_mut_ptr(), value);
    // NOTE: C++ appends sizeof(buf) of buf to string
    // NOTE: See 'put_fixed32' for explanation
    let l = dst.len();
    dst.push_str(&"\0".repeat(VAL_SIZE));
    unsafe {
        dst.as_mut_ptr()
            .add(l)
            .copy_from(buf.as_ptr().cast(), VAL_SIZE);
    }
}

fn put_varint32(dst: &mut String, v: u32) {
    // NOTE: Possibly use array and get ptr via arr.as_mut_slice().as_mut_ptr()
    let buf = &mut [0 as c_char; 5];
    let ptr = encode_varint32(buf.as_mut_ptr(), v);
    // NOTE: C++ appends (ptr - buf) of buf to string; shouldn't matter
    // NOTE: See 'put_fixed32' for explanation
    unsafe {
        let l = dst.len();
        let count = ptr.offset_from(buf.as_mut_ptr()) as usize;
        dst.push_str(&"\0".repeat(count));
        dst.as_mut_ptr()
            .add(l)
            .copy_from(buf.as_ptr().cast(), count);
    }
}

fn put_varint64(dst: &mut String, v: u64) {
    // NOTE: Possibly use array and get ptr via arr.as_mut_slice().as_mut_ptr()
    let buf = &mut [0 as c_char; 10];
    let ptr = encode_varint64(buf.as_mut_ptr(), v);
    // NOTE: C++ appends (ptr - buf) of buf to string; shouldn't matter
    // NOTE: See 'put_fixed32' for explanation
    unsafe {
        let l = dst.len();
        let count = ptr.offset_from(buf.as_mut_ptr()) as usize;
        dst.push_str(&"\0".repeat(count));
        dst.as_mut_ptr()
            .add(l)
            .copy_from(buf.as_ptr().cast(), count);
    }
}

fn put_length_prefixed_slice(dst: &mut String, value: &Slice) {
    put_varint32(dst, value.size() as u32);
    // NOTE: C++ appends value.size() of value.data() to string; shouldn't matter
    // NOTE: See 'put_fixed32' for explanation; possibly unneeded and can use line under

    // TODO: Test 2 versions below
    let l = dst.len();
    let count = value.size();
    dst.push_str(&"\0".repeat(count));
    unsafe {
        dst.as_mut_ptr()
            .add(l)
            .copy_from(value.data().cast(), count);
    }
    //dst.extend(value.to_string().chars())
}

// Standard Get.. routines parse a value from the beginning of a Slice and advance the slice past
// the parsed value.
fn get_varint32(input: &mut Slice, value: &mut u32) -> bool {
    let p = input.data();
    let limit = unsafe { p.add(input.size()) };
    let q = get_varint32_ptr(p, limit, value);
    // NOTE: In C++, nullptr possibly returned from func above and is checked below; here, None is
    // returned
    if let Some(q) = q {
        *input = unsafe { Slice::from_raw(q, limit.offset_from(q) as usize) };
        true
    } else {
        false
    }
}

fn get_varint64(input: &mut Slice, value: &mut u64) -> bool {
    let p = input.data();
    let limit = unsafe { p.add(input.size()) };
    let q = get_varint64_ptr(p, limit, value);
    // NOTE: In C++, nullptr possibly returned from func above and is checked below; here, None is
    // returned
    if let Some(q) = q {
        *input = unsafe { Slice::from_raw(p, limit.offset_from(q) as usize) };
        true
    } else {
        false
    }
}

fn get_length_prefixed_slice(input: &mut Slice, result: &mut Slice) -> bool {
    let mut len = 0;
    if get_varint32(input, &mut len) && input.size() >= len as usize {
        *result = Slice::from_raw(input.data(), len as usize);
        input.remove_prefix(len as usize);
        true
    } else {
        false
    }
}

// Pointer-based varints of GetVariant... These either store a value in &mut v and return a
// pointer just past the parsed value, or return None or error. These routines only look at bytes
// in the range [p..limit-1]
// NOTE: C++ returns nullptr rather than None
#[inline]
fn get_varint32_ptr(
    p: *const c_char,
    limit: *const c_char,
    value: &mut u32,
) -> Option<*const c_char> {
    if p < limit {
        unsafe {
            let result = *(p as *const u8) as u32;
            if result & 128 == 0 {
                *value = result;
                return Some(p.add(1));
            }
        }
    }
    get_varint32_ptr_fallback(p, limit, value)
}

// NOTE: Possibly make 'p' mutable in function
fn get_varint64_ptr(
    mut p: *const c_char,
    limit: *const c_char,
    v: &mut u64,
) -> Option<*const c_char> {
    let (mut result, mut shift) = (0, 0);
    while shift <= 63 && p < limit {
        let byte;
        unsafe {
            byte = *(p as *const u8) as u64;
            p = p.add(1);
        }
        if byte & 128 != 0 {
            // More bytes are present
            // NOTE: Remove extra parens?
            result |= (byte & 127) << shift;
        } else {
            // NOTE: Remove extra parens?
            result |= byte << shift;
            *v = result;
            // NOTE: Unneeded cast? (done in C++)
            return Some(p as *const c_char);
        }
        shift += 7;
    }
    None
}

// Returns the length of the varint32 or varint64 encoding of "v"
// NOTE: C++ returns int
// TODO: Possibly make mutable in function: let mut v = v
fn varint_length(mut v: u64) -> i32 {
    let mut len = 1;
    while v >= 128 {
        v >>= 7;
        len += 1;
    }
    len
}

// Lower-level versions of Put... that write directly into a character buffer and return a pointer
// just past the last byte written.
// REQUIRES: dst has enough space for the value being written
fn encode_varint32(dst: *mut c_char, v: u32) -> *mut c_char {
    // Operate on characters as unsigneds
    let mut ptr = dst as *mut u8;
    // NOTE: C++ uses int
    const B: u32 = 128;
    unsafe {
        if v < 1 << 7 {
            *ptr = v as u8;
            ptr = ptr.add(1);
        } else if v < 1 << 14 {
            *ptr = (v | B) as u8;
            ptr = ptr.add(1);
            *ptr = (v >> 7) as u8;
            ptr = ptr.add(1);
        } else if v < 1 << 21 {
            *ptr = (v | B) as u8;
            ptr = ptr.add(1);
            *ptr = ((v >> 7) | B) as u8;
            ptr = ptr.add(1);
            *ptr = (v >> 14) as u8;
            ptr = ptr.add(1);
        } else if v < 1 << 28 {
            *ptr = (v | B) as u8;
            ptr = ptr.add(1);
            *ptr = ((v >> 7) | B) as u8;
            ptr = ptr.add(1);
            *ptr = ((v >> 14) | B) as u8;
            ptr = ptr.add(1);
            *ptr = (v >> 21) as u8;
            ptr = ptr.add(1);
        } else {
            *ptr = (v | B) as u8;
            ptr = ptr.add(1);
            *ptr = ((v >> 7) | B) as u8;
            ptr = ptr.add(1);
            *ptr = ((v >> 14) | B) as u8;
            ptr = ptr.add(1);
            *ptr = ((v >> 21) | B) as u8;
            ptr = ptr.add(1);
            *ptr = (v >> 28) as u8;
            ptr = ptr.add(1);
        }
    }
    ptr as *mut c_char
}

// TODO: Possibly make mutable in function: let mut v = v
fn encode_varint64(dst: *mut c_char, mut v: u64) -> *mut c_char {
    // NOTE: C++ uses int
    const B: u64 = 128;
    let mut ptr = dst as *mut u8;
    while v >= B {
        unsafe {
            *ptr = (v | B) as u8;
            ptr = ptr.add(1);
        }
        v >>= 7;
    }
    unsafe {
        *ptr = v as u8;
        ptr = ptr.add(1);
    }
    ptr as *mut c_char
}

// Lower-level versions of Put.. that write directly into a character buffer
// REQUIRES: dst has enough space for the value being written
#[inline]
fn encode_fixed32(dst: *mut c_char, value: u32) {
    let buffer = dst as *mut u8;
    unsafe {
        *buffer = value as u8;
        *buffer.add(1) = (value >> 8) as u8;
        *buffer.add(2) = (value >> 16) as u8;
        *buffer.add(3) = (value >> 24) as u8;
    }
}

#[inline]
fn encode_fixed64(dst: *mut c_char, value: u64) {
    let buffer = dst as *mut u8;
    unsafe {
        *buffer = value as u8;
        *buffer.add(1) = (value >> 8) as u8;
        *buffer.add(2) = (value >> 16) as u8;
        *buffer.add(3) = (value >> 24) as u8;
        *buffer.add(4) = (value >> 32) as u8;
        *buffer.add(5) = (value >> 40) as u8;
        *buffer.add(6) = (value >> 48) as u8;
        *buffer.add(7) = (value >> 56) as u8;
    }
}

// Lower-level versions of Get... that read directly from a character buffer without any bounds
// checking.
#[inline]
pub(crate) fn decode_fixed32(ptr: *const c_char) -> u32 {
    let buffer = ptr as *const u8;
    unsafe {
        (*buffer as u32)
            | ((*buffer.add(1) as u32) << 8)
            | ((*buffer.add(2) as u32) << 16)
            | ((*buffer.add(3) as u32) << 24)
    }
}

#[inline]
fn decode_fixed64(ptr: *const c_char) -> u64 {
    let buffer = ptr as *const u8;
    unsafe {
        (*buffer as u64)
            | ((*buffer.add(1) as u64) << 8)
            | ((*buffer.add(2) as u64) << 16)
            | ((*buffer.add(3) as u64) << 24)
            | ((*buffer.add(4) as u64) << 32)
            | ((*buffer.add(5) as u64) << 40)
            | ((*buffer.add(6) as u64) << 48)
            | ((*buffer.add(7) as u64) << 56)
    }
}

// Internal routine for use by fallback path of GetVariant32Ptr
// NOTE: C++ returns nullptr instead of None
// TODO: Possibly make mutable in function: let mut p = p
fn get_varint32_ptr_fallback(
    mut p: *const c_char,
    limit: *const c_char,
    value: &mut u32,
) -> Option<*const c_char> {
    let (mut result, mut shift) = (0, 0);
    while shift <= 28 && p < limit {
        let byte;
        unsafe {
            byte = *(p as *const u8) as u32;
            p = p.add(1);
        }
        if byte & 128 != 0 {
            // More bytes are present
            // NOTE: Remove extra parens?
            result |= (byte & 127) << shift;
        } else {
            // NOTE: Remove extra parens?
            result |= byte << shift;
            *value = result;
            // NOTE: Unneeded cast? (done in C++)
            return Some(p as *const c_char);
        }
        shift += 7;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_fixed32() {
        let mut s = String::new();
        for v in 0..100_000 {
            put_fixed32(&mut s, v)
        }

        let mut p = s.as_ptr() as _;
        for v in 0..100_000 {
            let actual = decode_fixed32(p);
            assert_eq!(v, actual);
            unsafe {
                p = p.add(size_of::<u32>());
            }
        }
    }

    #[test]
    fn test_fixed64() {
        let mut s = String::new();
        for power in 0..63 {
            let v = 1 << power;
            put_fixed64(&mut s, v - 1);
            put_fixed64(&mut s, v + 0);
            put_fixed64(&mut s, v + 1);
        }

        let mut p = s.as_ptr() as _;
        for power in 0..63 {
            let v = 1 << power;
            let actual = decode_fixed64(p);
            assert_eq!(v - 1, actual);
            unsafe {
                p = p.add(size_of::<u64>());
            }

            let actual = decode_fixed64(p);
            assert_eq!(v + 0, actual);
            unsafe {
                p = p.add(size_of::<u64>());
            }

            let actual = decode_fixed64(p);
            assert_eq!(v + 1, actual);
            unsafe {
                p = p.add(size_of::<u64>());
            }
        }
    }

    #[test]
    fn test_encoding_output() {
        let mut dst = String::new();
        put_fixed32(&mut dst, 0x04030201);
        assert_eq!(4, dst.len());
        // NOTE: Should be the same as dereferencing the raw underlying pointer
        assert_eq!(0x01, dst.as_bytes()[0] as i32);
        assert_eq!(0x02, dst.as_bytes()[1] as i32);
        assert_eq!(0x03, dst.as_bytes()[2] as i32);
        assert_eq!(0x04, dst.as_bytes()[3] as i32);

        dst.clear();
        put_fixed64(&mut dst, 0x0807060504030201);
        assert_eq!(8, dst.len());
        assert_eq!(0x01, dst.as_bytes()[0] as i32);
        assert_eq!(0x02, dst.as_bytes()[1] as i32);
        assert_eq!(0x03, dst.as_bytes()[2] as i32);
        assert_eq!(0x04, dst.as_bytes()[3] as i32);
        assert_eq!(0x05, dst.as_bytes()[4] as i32);
        assert_eq!(0x06, dst.as_bytes()[5] as i32);
        assert_eq!(0x07, dst.as_bytes()[6] as i32);
        assert_eq!(0x08, dst.as_bytes()[7] as i32);
    }

    #[test]
    fn test_varint32() {
        let mut s = String::new();
        for i in 0..32 * 32 {
            let v = (i / 32) << (i % 32);
            put_varint32(&mut s, v);
        }

        unsafe {
            let mut p = s.as_ptr() as *const c_char;
            let limit = p.add(s.len());
            for i in 0..32 * 32 {
                let expected = (i / 32) << (i % 32);
                let mut actual = 0;
                let start = p;
                // NOTE: C++ returns nullptr but here None is returned instead
                let op = get_varint32_ptr(p, limit, &mut actual);
                assert!(op.is_some());
                p = op.unwrap();
                assert_eq!(expected, actual);
                assert_eq!(varint_length(actual as u64), p.offset_from(start) as i32);
            }
            assert_eq!(p, s.as_ptr().cast::<c_char>().add(s.len()));
        }
    }

    #[test]
    fn test_varint64() {
        // Construct the list of values to check
        let mut values = Vec::new();
        // Some special values
        values.push(0);
        values.push(100);
        values.push(!0u64);
        values.push(!0u64 - 1);
        for k in 0..64 {
            // Test values near powers of two
            let power = 1 << k;
            values.push(power);
            values.push(power - 1);
            values.push(power + 1);
        }

        let mut s = String::new();
        for i in 0..values.len() {
            put_varint64(&mut s, values[i]);
        }

        unsafe {
            let mut p = s.as_ptr() as *const c_char;
            let limit = p.add(s.len());
            for i in 0..values.len() {
                assert!(p < limit);
                let mut actual = 0;
                let start = p;
                // NOTE: C++ returns nullptr but here None is returned instead
                let op = get_varint64_ptr(p, limit, &mut actual);
                assert!(op.is_some());
                p = op.unwrap();
                assert_eq!(values[i], actual);
                assert_eq!(varint_length(actual), p.offset_from(start) as i32);
            }
            assert_eq!(p, limit);
        }
    }

    #[test]
    fn test_varint32_overflow() {
        let mut result = 0;
        // NOTE: Use unicode since the max hex literal is 0x7f (0o177, 127)
        let input = String::from("\u{81}\u{82}\u{83}\u{84}\u{85}\u{11}");
        unsafe {
            let p = input.as_ptr() as *const c_char;
            assert!(
                // NOTE: C++ returns nullptr but here None is returned instead
                get_varint32_ptr(p, p.add(input.len()), &mut result,).is_none(),
            );
        }
    }

    #[test]
    fn test_varint32_truncation() {
        let large_value = (1 << 31) + 100;
        let mut s = String::new();
        put_varint32(&mut s, large_value);
        let mut result = 0;
        let p = s.as_ptr() as *const c_char;
        unsafe {
            for len in 0..s.len() - 1 {
                // NOTE: C++ returns nullptr but here None is returned instead
                assert!(get_varint32_ptr(p, p.add(len), &mut result).is_none());
            }
            assert!(get_varint32_ptr(p, p.add(s.len()), &mut result).is_some());
        }
        assert_eq!(large_value, result);
    }

    #[test]
    fn test_varint64_overflow() {
        let mut result = 0;
        // NOTE: Use unicode since the max hex literal is 0x7f (0o177, 127)
        let input =
            String::from("\u{81}\u{82}\u{83}\u{84}\u{85}\u{81}\u{82}\u{83}\u{84}\u{85}\u{11}");
        unsafe {
            let p = input.as_ptr() as *const c_char;
            assert!(
                // NOTE: C++ returns nullptr but here None is returned instead
                get_varint64_ptr(p, p.add(input.len()), &mut result,).is_none(),
            );
        }
    }

    #[test]
    fn test_varint64_truncation() {
        let large_value = (1 << 63) + 100;
        let mut s = String::new();
        put_varint64(&mut s, large_value);
        let mut result = 0;
        let p = s.as_ptr() as *const c_char;
        unsafe {
            for len in 0..s.len() - 1 {
                // NOTE: C++ returns nullptr but here None is returned instead
                assert!(get_varint64_ptr(p, p.add(len), &mut result).is_none());
            }
            assert!(get_varint64_ptr(p, p.add(s.len()), &mut result).is_some());
        }
        assert_eq!(large_value, result);
    }

    #[test]
    fn test_strings() {
        let mut s = String::new();
        put_length_prefixed_slice(&mut s, &Slice::from(""));
        put_length_prefixed_slice(&mut s, &Slice::from("foo"));
        put_length_prefixed_slice(&mut s, &Slice::from("bar"));
        put_length_prefixed_slice(&mut s, &Slice::from(&"x".repeat(200)));

        let mut input = Slice::from(&s);
        let mut v = Slice::new();
        assert!(get_length_prefixed_slice(&mut input, &mut v));
        assert_eq!("", v.to_string());
        assert!(get_length_prefixed_slice(&mut input, &mut v));
        assert_eq!("foo", v.to_string());
        assert!(get_length_prefixed_slice(&mut input, &mut v));
        assert_eq!("bar", v.to_string());
        assert!(get_length_prefixed_slice(&mut input, &mut v));
        assert_eq!("x".repeat(200), v.to_string());
        assert_eq!("", input.to_string());
    }
}
