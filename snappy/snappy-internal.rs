// snappy/snappy-internal.h

use std::arch::asm;

#[path = "snappy-stubs-internal.rs"]
mod snappy_stubs_internal;
use snappy_stubs_internal::{
    Bits, LittleEndian, SNAPPY_PREDICT_TRUE, UNALIGNED_LOAD32, UNALIGNED_LOAD64,
};

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "ssse3",
    target_feature = "sse2"
))]
pub(crate) mod internal {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    type V128 = __m128i;

    // Load 128 bits of integer data. `src` must be 16-byte aligned.
    // NOTE: C++ uses const*
    #[inline]
    pub(crate) fn v128_load(src: &V128) -> V128 {
        unsafe { _mm_load_si128(src as *const _) }
    }

    // Load 128 bits of integer data. `src` does not need to be aligned.
    #[inline]
    pub(crate) fn v128_loadu(src: &V128) -> V128 {
        unsafe { _mm_loadu_si128(src as *const _) }
    }

    // Store 128 bits of integer data. `dst` does not need to be aligned.
    #[inline]
    pub(crate) fn v128_storeu(dst: &mut V128, val: V128) {
        unsafe {
            _mm_storeu_ui128(dst as *mut _, val);
        }
    }

    // Shuffle packed 8-bit integers using shuffle mask.
    // Each packed integer in the shuffle mask must be in [0,16).
    #[inline]
    pub(crate) fn v128_shuffle(input: V128, shuffle_mask: V128) -> V128 {
        unsafe { _mm_shuffle_epi8(input, shuffle_mask) }
    }

    // Constructs V128 with 16 chars (bytes) |c|.
    #[inline]
    pub(crate) fn v128_dup_char(c: u8) -> V128 {
        unsafe { _mm_set1_epi8(c as i8) }
    }
}

#[cfg(
    all(
        target_arch = "AArch64"
        target_feature = "neon"
    )
)]
pub(crate) mod internal {
    use std::arch::aarch64::*;

    type V128 = uint8x16_t;

    // Load 128 bits of integer data. `src` must be 16-byte aligned.
    // NOTE: C++ uses const*
    #[inline]
    pub(crate) fn v128_load(src: &V128) -> V128 {
        unsafe { vld1q_u8(src as *const _ as *const _) }
    }

    // Load 128 bits of integer data. `src` does not need to be aligned.
    #[inline]
    pub(crate) fn v128_loadu(src: &V128) -> V128 {
        unsafe { vld1q_u8(src as *const _ as *const _) }
    }

    // Store 128 bits of integer data. `dst` does not need to be aligned.
    #[inline]
    pub(crate) fn v128_storeu(dst: &mut V128, val: V128) {
        unsafe {
            vst1q_u8(dst as *mut _ as *mut _, val);
        }
    }

    // Shuffle packed 8-bit integers using shuffle mask.
    // Each packed integer in the shuffle mask must be in [0,16).
    #[inline]
    pub(crate) fn v128_shuffle(input: V128, shuffle_mask: V128) -> V128 {
        assert!(vminvq_u8(shuffle_mask) >= 0 && vmaxvq_u8(shuffle_mask) <= 15);
        unsafe { vqtbl1q_u8(input, shuffle_mask) }
    }

    // Constructs V128 with 16 chars (bytes) |c|.
    #[inline]
    pub(crate) fn v128_dup_char(c: u8) -> V128 {
        unsafe { vdupq_n_u8(c) }
    }
}

use internal::*;

// Working memory performs a single allocation to hold all scratch space required for compression.
pub(crate) struct WorkingMemory {
    mem: *mut u8,    // The allocated memory, never null
    size: usize,     // The size of the allocated memory, never 0
    table: *mut u16, // The pointer to the hashtable
    input: *mut u8,  // THhe pointer to the input scratch buffer
    output: *mut u8, // THhe pointer to the output scratch buffer
}

impl WorkingMemory {
    pub(crate) fn new(input_size: usize) -> Self {
        //
    }

    // Allocated and clears a hash table using memory in "self", stores the number of buckets in
    // "table_size", and returns a pointer to the base of the hash table.
    // NOTE: C++ uses int*
    // TODO: Return table_size as well?
    pub(crate) fn get_hash_table(&self, fragment_size: usize, table_size: &mut i32) -> *mut u16;

    pub(crate) fn get_scratch_input(&self) -> *mut u8 {
        self.input
    }

    pub(crate) fn get_scratch_output(&self) -> *mut u8 {
        self.output
    }
}

// Flat array compression that does not emit the "uncompressed length" prefix. Compresses "input"
// string to the "op" buffer.
//
// REQUIRES: "input_length <= BLOCK_SIZE"
// REQUIRES: "op" points to an array of memory at least "max_compressed_length(input_length)" in
// size.
// REQUIRES: All elements in "table[0..table_size-1]" are initialized to zero.
//
// Returns an "end" pointer into "op" buffer.
// "end - op" is the compressed size of "input".
// NOTE: C++ uses int
pub(crate) fn compress_fragment(
    input: *const u8,
    input_length: usize,
    op: *mut u8,
    table: *mut u16,
    table_size: i32,
) -> *mut u8;

// Find the largest n such that
//
//   s1[0,n-1] == s2[0,n-1]
//   and n <= (s2_limit - s2).
//
// Return (n, n < 8).
// Does not read *s2_limit or beyond.
// Does not read *(s1 + (s2_limit - s2)) or beyond.
// Requires that s2_limit >= 2.
//
// In addition populate "data" with the next 5 bytes from the end of the match. This is only done
// if 8 bytes are available (s2_limit - s2 >= 8). The point is that on some arch's this can be
// done faster in this routine than subsequent loading from s2 + n.
//
// Separate implementation for 64-bit, little-endian cpus.
// NOTE: C++ uses u64*
// TODO: C++ ifdefs
#[inline]
fn find_match_length(
    s1: *const u8,
    mut s2: *const u8,
    s2_limit: *const u8,
    data: &mut u64,
) -> (usize, bool) {
    assert!(s2_limit >= s2);
    let mut matched = 0usize;

    // This block isn't necessary for correctness; we could just start looping immediately. As an
    // optimization though, it is useful. It creates some not uncommon code paths that determine,
    // without extra effort, whether the match length is less than 8. In short, we are hoping to
    // avoid a conditional branch, and perhaps get better code layout from the C++ compiler.
    if SNAPPY_PREDICT_TRUE(s2 <= unsafe { s2_limit.sub(16) }) {
        let a1 = UNALIGNED_LOAD64(s1);
        let a2 = UNALIGNED_LOAD64(s2);
        if SNAPPY_PREDICT_TRUE(a1 != a2) {
            // This code is critical for performance. The reason is that it determines how much to
            // advance `ip` (s2). This obviously depends on both the loads from the `candidate`
            // (s1) and `ip`. Furthermore the next `candidate` depends on the advanced `ip`
            // calculated here through a load, hash and new candidate hash lookup (a lot of
            // cycles). This makes s1 (ie. `candidate`) the variable that limits throughput. This
            // is the reason we go through hoops to have this function update `data` for the next
            // iter. The straightforward code would use *data, given by
            //
            // *data = UNALIGNED_LOAD64(s2 + matched_bytes) (Latency of 5 cycles),
            //
            // as input for the hash table lookup to find next candidate. However this forces the
            // load on the data dependency chain of s1, because matched_bytes directly depends on
            // s1. However matched_bytes is 0..7, so we can also calculate *data by
            //
            // *data = AlignRight(UNALIGNED_LOAD64(s2), UNALIGNED_LOAD64(s2 + 8),
            //                    matched_bytes);
            //
            // The loads do not depend on s1 anymore and are thus off the bottleneck. The
            // straightforward implementation on x86_64 would be to use
            //
            // shrd rax, rdx, cl  (cl being matched_bytes * 8)
            //
            // unfortunately shrd with a variable shift has a 4 cycle latency. So this only wins 1
            // cycle. The BMI2 shrx instruction is a 1 cycle variable shift instruction but can
            // only shift 64 bits. If we focus on just obtaining the least significant 4 bytes, we
            // can obtain this by
            //
            // *data = ConditionalMove(matched_bytes < 4, UNALIGNED_LOAD64(s2),
            //     UNALIGNED_LOAD64(s2 + 4) >> ((matched_bytes & 3) * 8);
            //
            // Writen like above this is not a big win, the conditional move would be a cmp
            // followed by a cmov (2 cycles) followed by a shift (1 cycle). However
            // matched_bytes < 4 is equal to
            // static_cast<uint32_t>(xorval) != 0. Writen that way, the conditional move (2
            // cycles) can execute in parallel with FindLSBSetNonZero64 (tzcnt), which takes 3
            // cycles.
            // TODO: Does this still hold (above)?
            let xorval = a1 ^ a2;
            let shift = Bits::find_lsb_set_non_zero64(xorval);
            let matched_bytes = (shift >> 3) as usize;
            #[cfg(not(target_arch = "x86-64"))]
            *data = UNALIGNED_LOAG64(unsafe { s2.add(matched_bytes) });
            #[cfg(target_arch = "x86-64")]
            unsafe {
                // Ideally this would just be
                //
                // (In C++) a = static_cast<uint32_t>(xorval) == 0 ? a3 : a2;)
                //
                // However clang correctly infers that the above statement participates on a critical
                // data dependency chain and thus, unfortunately, refuses to use a conditional move
                // (it's tuned to cut data dependencies). In this case there is a longer parallel
                // chain anyway AND this will be fairly unpredictable.
                // TODO: Does this still hold (above)?
                let a3 = UNALIGNED_LOAD64(s2.add(4));
                // TODO
                asm!("testl %k2, %k2",
                    "cmovzq {1}, {0}",
                    out(a2),
                    in(a3), in(xorval),
                );
                *data = a2 >> (shift & (3 * 8)) as usize;
            }
            return (matched_bytes, true);
        } else {
            matched = 8;
            unsafe {
                s2 = s2.add(8);
            }
        }
    }

    // Find out how long the match is. We loop over the data 64 bits at a time until we find a
    // 64-bit block that doesn't match; then we find the first non-matching bit and use that to
    // calculate the total length of the match.
    unsafe {
        while SNAPPY_PREDICT_TRUE(s2 <= s2_limit.sub(16)) {
            let a1 = UNALIGNED_LOAD64(s1.add(matched));
            let a2 = UNALIGNED_LOAD64(s2);
            if a1 == a2 {
                s2 = s2.add(8);
                matched += 8;
            } else {
                let xorval = a1 ^ a2;
                let shift = Bits::find_lsb_set_non_zero64(xorval);
                let matched_bytes = (shift >> 3) as usize;
                #[cfg(not(target_arch = "x86-64"))]
                *data = UNALIGNED_LOAG64(unsafe { s2.add(matched_bytes) });
                #[cfg(target_arch = "x86-64")]
                {
                    let a3 = UNALIGNED_LOAD64(s2.add(4));
                    // TODO
                    asm!("testl %k2, %k2",
                        "cmovzq {1}, {0}",
                        out(a2),
                        in(a3), in(xorval),
                    );
                    *data = a2 >> (shift & (3 * 8)) as usize;
                }
                matched += matched_bytes;
                assert!(matched >= 8);
                return (matched, false);
            }
        }
        while SNAPPY_PREDICT_TRUE(s2 < s2_limit) {
            if *s1.add(matched) == *s2 {
                s2 = s2.add(1);
                matched += 1;
            } else {
                if s2 <= s2_limit.sub(8) {
                    *data = UNALIGNED_LOAD64(s2);
                }
                return (matched, matched < 8);
            }
        }
        (matched, matched < 8)
    }
}

#[inline]
fn find_match_length(
    s1: *const u8,
    mut s2: *const u8,
    s2_limit: *const u8,
    data: &mut u64,
) -> (usize, bool) {
    // Implementation based on the x86-64 version, above.
    assert!(s2_limit >= s2);
    // NOTE: C++ uses int
    let mut matched = 0usize;

    unsafe {
        while s2 <= s2_limit.sub(4) && UNALIGNED_LOAD32(s2) == UNALIGNED_LOAD32(s1.add(matched)) {
            s2 = s2.add(4);
            matched += 4;
        }
        if LittleEndian::is_little_endian() && s2 <= s2_limit.sub(4) {
            let mut x = UNALIGNED_LOAD32(s2) ^ UNALIGNED_LOAD32(s1.add(matched));
            let matching_bits = Bits::find_lsb_set_non_zero(x);
            matched += (matching_bits >> 3) as usize;
            s2 = s2.add((matching_bits >> 3) as usize);
        } else {
            while s2 < s2_limit && *s1.add(matched) == *s2 {
                s2 = s2.add(1);
                matched += 1;
            }
        }
        if s2 <= s2_limit.sub(8) {
            *data = LittleEndian::load64(s2);
        }
    }
    (matched, matched < 8)
}

// Lookup tables for decompression code. Give --snappy_dump_decompression_table to the unit test
// to recompute char_table.
// TODO: Change above accordingly

// NOTE: C++ has anonymous (untagged) enum and should be used as follows:
// TODO: Use const?
// LITERAL as i32 (int)
#[repr(C)]
pub(crate) enum __UNTAGGED {
    LITERAL = 0,
    COPY_1_BYTE_OFFSET = 1,
    COPY_2_BYTE_OFFSET = 2,
    COPY_3_BYTE_OFFSET = 3,
}
pub(crate) use __UNTAGED::*;
// NOTE: C++ uses int
const MAXIMUM_TAG_LENGTH: i32 = 5; // COPY_4_BYTE_OFFSET plus the actual offset.

// Data stored per entry in lookup table:
//      Range   Bits-used       Description
//      -----------------------------------
//      1..64   0..7            Literal/copy length encoded in opcode byte
//      1..7    8..10           Copy offset encoded in opcode byte / 256
//      0..4    11..13          Extra bytes after opcode
//
// We use eight bits for the length even though 7 would have sufficed because of efficiency
// reasons:
//      (1) Extracting a byte is faster than a bit-field
//      (2) It properly aligns copy offset so we do not need a <<8
const char_table: [u16; 256] = [
    0x0001, 0x0804, 0x1001, 0x2001, 0x0002, 0x0805, 0x1002, 0x2002, 0x0003, 0x0806, 0x1003, 0x2003,
    0x0004, 0x0807, 0x1004, 0x2004, 0x0005, 0x0808, 0x1005, 0x2005, 0x0006, 0x0809, 0x1006, 0x2006,
    0x0007, 0x080a, 0x1007, 0x2007, 0x0008, 0x080b, 0x1008, 0x2008, 0x0009, 0x0904, 0x1009, 0x2009,
    0x000a, 0x0905, 0x100a, 0x200a, 0x000b, 0x0906, 0x100b, 0x200b, 0x000c, 0x0907, 0x100c, 0x200c,
    0x000d, 0x0908, 0x100d, 0x200d, 0x000e, 0x0909, 0x100e, 0x200e, 0x000f, 0x090a, 0x100f, 0x200f,
    0x0010, 0x090b, 0x1010, 0x2010, 0x0011, 0x0a04, 0x1011, 0x2011, 0x0012, 0x0a05, 0x1012, 0x2012,
    0x0013, 0x0a06, 0x1013, 0x2013, 0x0014, 0x0a07, 0x1014, 0x2014, 0x0015, 0x0a08, 0x1015, 0x2015,
    0x0016, 0x0a09, 0x1016, 0x2016, 0x0017, 0x0a0a, 0x1017, 0x2017, 0x0018, 0x0a0b, 0x1018, 0x2018,
    0x0019, 0x0b04, 0x1019, 0x2019, 0x001a, 0x0b05, 0x101a, 0x201a, 0x001b, 0x0b06, 0x101b, 0x201b,
    0x001c, 0x0b07, 0x101c, 0x201c, 0x001d, 0x0b08, 0x101d, 0x201d, 0x001e, 0x0b09, 0x101e, 0x201e,
    0x001f, 0x0b0a, 0x101f, 0x201f, 0x0020, 0x0b0b, 0x1020, 0x2020, 0x0021, 0x0c04, 0x1021, 0x2021,
    0x0022, 0x0c05, 0x1022, 0x2022, 0x0023, 0x0c06, 0x1023, 0x2023, 0x0024, 0x0c07, 0x1024, 0x2024,
    0x0025, 0x0c08, 0x1025, 0x2025, 0x0026, 0x0c09, 0x1026, 0x2026, 0x0027, 0x0c0a, 0x1027, 0x2027,
    0x0028, 0x0c0b, 0x1028, 0x2028, 0x0029, 0x0d04, 0x1029, 0x2029, 0x002a, 0x0d05, 0x102a, 0x202a,
    0x002b, 0x0d06, 0x102b, 0x202b, 0x002c, 0x0d07, 0x102c, 0x202c, 0x002d, 0x0d08, 0x102d, 0x202d,
    0x002e, 0x0d09, 0x102e, 0x202e, 0x002f, 0x0d0a, 0x102f, 0x202f, 0x0030, 0x0d0b, 0x1030, 0x2030,
    0x0031, 0x0e04, 0x1031, 0x2031, 0x0032, 0x0e05, 0x1032, 0x2032, 0x0033, 0x0e06, 0x1033, 0x2033,
    0x0034, 0x0e07, 0x1034, 0x2034, 0x0035, 0x0e08, 0x1035, 0x2035, 0x0036, 0x0e09, 0x1036, 0x2036,
    0x0037, 0x0e0a, 0x1037, 0x2037, 0x0038, 0x0e0b, 0x1038, 0x2038, 0x0039, 0x0f04, 0x1039, 0x2039,
    0x003a, 0x0f05, 0x103a, 0x203a, 0x003b, 0x0f06, 0x103b, 0x203b, 0x003c, 0x0f07, 0x103c, 0x203c,
    0x0801, 0x0f08, 0x103d, 0x203d, 0x1001, 0x0f09, 0x103e, 0x203e, 0x1801, 0x0f0a, 0x103f, 0x203f,
    0x2001, 0x0f0b, 0x1040, 0x2040,
];
