// crc32c/{include/crc32c/crc32c.h,src/crc32c.cc}

pub fn extend(crc: u32, data: *const u8, count: usize) -> u32 {
    0
}

pub fn crc32c_extend(crc: u32, data: *const u8, count: usize) -> u32 {
    extend(0, data, count)
}

pub fn crc32c_value(data: *const u8, count: usize) -> u32 {
    crc32c(data, count)
}
