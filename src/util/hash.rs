// leveldb/util/hash.{h,cc}
// leveldb/util/hash_test.cc

use std::os::raw::c_char;

// NOTE: Make mutable in function: let mut data = data?
pub fn hash(mut data: *const c_char, n: usize, seed: u32) -> u32 {
    // Similar to murmur hash
    const M: u32 = 0xc6a4a793;
    const R: u32 = 24;
    let limit = unsafe { data.add(n) };
    let mut h = seed ^ (n as u32).wrapping_mul(M);

    unsafe {
        // Pick up four bytes at a time
        while data.add(4) <= limit {
            let w = crate::util::coding::decode_fixed32(data);
            data = data.add(4);
            h = h.wrapping_add(w);
            h = h.wrapping_mul(M);
            h ^= h >> 16;
        }

        // Pick up remaining bytes
        // NOTE: Switch w/ fallthrough in C++
        let mut remaining = limit.offset_from(data) as usize;
        if remaining == 3 {
            h += ((*data.add(2)) as u8 as u32) << 16;
            remaining -= 1; // Emulate fallthrough
        }
        if remaining == 2 {
            h += ((*data.add(1)) as u8 as u32) << 8;
            remaining -= 1; // Emulate fallthrough
        }
        if remaining == 1 {
            h = h.wrapping_add((*data.add(0)) as u8 as u32);
            h = h.wrapping_mul(M);
            h ^= h >> R;
        }
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr::null;

    #[test]
    fn test_signed_unsigned_issue() {
        const DATA1: [u8; 1] = [0x62];
        const DATA2: [u8; 2] = [0xc3, 0x97];
        const DATA3: [u8; 3] = [0xe2, 0x99, 0xa5];
        const DATA4: [u8; 4] = [0xe1, 0x80, 0xb9, 0x32];
        const DATA5: [u8; 48] = [
            0x01, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x14,
            0x00, 0x00, 0x00, 0x18, 0x28, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        assert_eq!(hash(null(), 0, 0xbc9f1d34), 0xbc9f1d34);
        assert_eq!(
            hash(DATA1.as_ptr().cast(), DATA1.len(), 0xbc9f1d34),
            0xef1345c4
        );
        assert_eq!(
            hash(DATA2.as_ptr().cast(), DATA2.len(), 0xbc9f1d34),
            0x5b663814
        );
        assert_eq!(
            hash(DATA3.as_ptr().cast(), DATA3.len(), 0xbc9f1d34),
            0x323c078f
        );
        assert_eq!(
            hash(DATA4.as_ptr().cast(), DATA4.len(), 0xbc9f1d34),
            0xed21633a
        );
        assert_eq!(
            hash(DATA5.as_ptr().cast(), DATA5.len(), 0x12345678),
            0xf333dabb
        );
    }
}
