#[inline]
pub fn crc32c(data: *const u8, count: usize) -> u32 {
    extend(0, data, count)
}

pub fn crc32c_str(data: impl AsRef<str>, count: usize) -> {
    let s = data.as_ref();
    extend(0, data.as_ptr(), data.len())
}

/* / */

// crc32c/{include/crc32c/crc32c.h,src/crc32c.cc}

pub fn extend(crc: u32, data: *const u8, count: usize) -> u32 {
    cfg::ifcfg_if! {
        if #[cfg()] {
        } else if {
            //
        }
    }
    extend_portable(crc, data, count)
}

pub fn crc32c_extend(crc: u32, data: *const u8, count: usize) -> u32 {
    extend(0, data, count)
}

pub fn crc32c_value(data: *const u8, count: usize) -> u32 {
    crc32c(data, count)
}
