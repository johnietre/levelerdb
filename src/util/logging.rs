// leveldb/util/logging.{h,cc}
// leveldb/util/logging_test.cc

#![allow(dead_code)]
use crate::slice::Slice;

// TODO: From port?
struct WritableFile {}

// Append a human-readable printout of 'num' to 's'.
// NOTE: C++ uses pointer rather than mutable reference
pub fn append_number_to(s: &mut String, num: u64) {
    s.push_str(&num.to_string());
}

// Append a human-readable printout of 'value' to 's'.
// Escapes any non-printable characters found in 'value'.
// NOTE: C++ uses pointer rather than mutable reference
pub fn append_escaped_string_to(s: &mut String, value: &Slice) {
    for i in 0..value.size() {
        let c = value[i] as char;
        if c >= ' ' && c <= '~' {
            s.push(c);
        } else {
            s.push_str(&format!("\\x{:02x}", c as u32 & 0xff,));
        }
    }
}

// Return a human-readable printout of 'num'.
pub fn number_to_string(num: u64) -> String {
    /* NOTE: C++ uses lines below
    let mut s = String::new();
    append_number_to(&mut s, num)
    s
    */
    num.to_string()
}

// Return a human-readable version of 'value'.
// Escapes any non-printable characters found in 'value'.
pub fn escape_string(value: &Slice) -> String {
    let mut r = String::with_capacity(value.size());
    append_escaped_string_to(&mut r, value);
    r
}

// Parse a human-readable number form 'in_slice' into 'val'. On success, advances 'in_slice' past
// the consumed number and sets 'val' to the numeric value. Otherwise, returns false and leaves
// 'in_slice' in an unspecified state.
// NOTE: Both mutable references here are pointers in C++
pub fn consume_decimal_number(in_slice: &mut Slice, val: &mut u64) -> bool {
    // Constants that will be optimized away.
    const MAX_U64: u64 = u64::MAX;
    // Only "u8" can cast to "char" so the value must be cast to "u8" first
    // NOTE: C++ uses "char" ("c_char")
    const LAST_DIGIT_OF_MAX_U64: u8 = (('0' as u64) + (MAX_U64 % 10)) as u8;
    let mut value = 0;

    // Cast from *const c_char to *const u8 to avoid signedness
    let start = in_slice.data() as *const u8;
    let end = unsafe { start.add(in_slice.size()) };
    let mut current = start;
    while current != end {
        let ch = unsafe { *current };
        if ch < b'0' || ch > b'9' {
            break;
        }
        // Overflow check.
        // MAX_U64 / 10 is also constant and will be optimized away.
        if value > MAX_U64 / 10 || (value == MAX_U64 / 10 && ch > LAST_DIGIT_OF_MAX_U64) {
            return false;
        }
        value = (value * 10) + (ch - b'0') as u64;
        current = unsafe { current.add(1) };
    }

    *val = value;
    let digits_consumed = unsafe { current.offset_from(start) as usize };
    in_slice.remove_prefix(digits_consumed);
    digits_consumed != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_to_string() {
        assert_eq!("0", number_to_string(0));
        assert_eq!("1", number_to_string(1));
        assert_eq!("9", number_to_string(9));

        assert_eq!("10", number_to_string(10));
        assert_eq!("11", number_to_string(11));
        assert_eq!("19", number_to_string(19));
        assert_eq!("99", number_to_string(99));

        assert_eq!("100", number_to_string(100));
        assert_eq!("109", number_to_string(109));
        assert_eq!("190", number_to_string(190));
        assert_eq!("123", number_to_string(123));
        assert_eq!("12345678", number_to_string(12345678));

        assert_eq!(u64::MAX, 18446744073709551615, "Test consistency check");
        assert_eq!(
            "18446744073709551000",
            number_to_string(18446744073709551000)
        );
        assert_eq!(
            "18446744073709551600",
            number_to_string(18446744073709551600)
        );
        assert_eq!(
            "18446744073709551610",
            number_to_string(18446744073709551610)
        );
        assert_eq!(
            "18446744073709551614",
            number_to_string(18446744073709551614)
        );
        assert_eq!(
            "18446744073709551615",
            number_to_string(18446744073709551615)
        );
    }

    fn consume_decimal_number_roundtrip_test(number: u64, padding: &str) {
        let decimal_number = number_to_string(number);
        let input_string = decimal_number.clone() + padding;
        let input = Slice::from(&input_string);
        let mut output = input;
        let mut result = 0;
        assert!(consume_decimal_number(&mut output, &mut result));
        assert_eq!(number, result);
        assert_eq!(decimal_number.len(), unsafe {
            output.data().offset_from(input.data()) as usize
        },);
        assert_eq!(padding.len(), output.size());
    }

    #[test]
    fn test_consume_decimal_number_roundtrip() {
        consume_decimal_number_roundtrip_test(0, "");
        consume_decimal_number_roundtrip_test(1, "");
        consume_decimal_number_roundtrip_test(9, "");

        consume_decimal_number_roundtrip_test(10, "");
        consume_decimal_number_roundtrip_test(11, "");
        consume_decimal_number_roundtrip_test(19, "");
        consume_decimal_number_roundtrip_test(99, "");

        consume_decimal_number_roundtrip_test(100, "");
        consume_decimal_number_roundtrip_test(109, "");
        consume_decimal_number_roundtrip_test(190, "");
        consume_decimal_number_roundtrip_test(123, "");
        assert_eq!("12345678", number_to_string(12345678));

        for i in 0..100 {
            let large_number = u64::MAX - i;
            consume_decimal_number_roundtrip_test(large_number, "");
        }
    }

    #[test]
    fn test_consume_decimal_number_roundtrip_with_padding() {
        consume_decimal_number_roundtrip_test(0, " ");
        consume_decimal_number_roundtrip_test(1, "abc");
        consume_decimal_number_roundtrip_test(9, "x");

        consume_decimal_number_roundtrip_test(10, "-");
        // NOTE: Possibly unneeded since Rust strings aren't null terminated
        consume_decimal_number_roundtrip_test(11, &"\0\0\0".repeat(3));
        consume_decimal_number_roundtrip_test(19, "abc");
        consume_decimal_number_roundtrip_test(99, "padding");

        consume_decimal_number_roundtrip_test(100, " ");

        for i in 0..100 {
            let large_number = u64::MAX - i;
            consume_decimal_number_roundtrip_test(large_number, "pad");
        }
    }

    fn consume_decimal_number_overflow_test(input_string: &str) {
        let input = Slice::from(input_string);
        let mut output = input;
        let mut result = 0;
        assert_eq!(false, consume_decimal_number(&mut output, &mut result));
    }

    #[test]
    fn test_consume_decimal_number_overflow() {
        assert_eq!(u64::MAX, 18446744073709551615, "Test consistency check");
        consume_decimal_number_overflow_test("18446744073709551616");
        consume_decimal_number_overflow_test("18446744073709551617");
        consume_decimal_number_overflow_test("18446744073709551618");
        consume_decimal_number_overflow_test("18446744073709551619");
        consume_decimal_number_overflow_test("18446744073709551620");
        consume_decimal_number_overflow_test("18446744073709551621");
        consume_decimal_number_overflow_test("18446744073709551622");
        consume_decimal_number_overflow_test("18446744073709551623");
        consume_decimal_number_overflow_test("18446744073709551624");
        consume_decimal_number_overflow_test("18446744073709551625");
        consume_decimal_number_overflow_test("18446744073709551626");

        consume_decimal_number_overflow_test("18446744073709551700");

        consume_decimal_number_overflow_test("99999999999999999999");
    }

    fn consume_decimal_number_no_digits_test(input_string: &str) {
        let input = Slice::from(input_string);
        let mut output = input;
        let mut result = 0;
        assert_eq!(false, consume_decimal_number(&mut output, &mut result));
        assert_eq!(input.data(), output.data());
        assert_eq!(input.size(), output.size());
    }

    #[test]
    fn test_consume_decimal_number_no_digits() {
        consume_decimal_number_no_digits_test("");
        consume_decimal_number_no_digits_test(" ");
        consume_decimal_number_no_digits_test("a");
        consume_decimal_number_no_digits_test(" 123");
        consume_decimal_number_no_digits_test("a123");
        // NOTE: Rust doesn't allow octal literals in strings?
        // Also, the escaped 255 yields a string of length 1 in C++ byt 2 in Rust, which may
        // produce different results from the test
        // NOTE: The above doesn't seem to matter
        consume_decimal_number_no_digits_test(&"\x00123".repeat(4));
        consume_decimal_number_no_digits_test(&"\x7f123".repeat(4));
        // NOTE: Use unicode since the max hex literal is 0x7f (0o177, 127)
        consume_decimal_number_no_digits_test(&"\u{FF}0123".repeat(4));
    }
}
