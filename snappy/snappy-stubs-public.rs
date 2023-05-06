#[repr(C)]
pub struct iovec {
    iov_base: *mut std::ffi::c_void,
    iov_len: usize,
}
