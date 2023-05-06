// snappy/snappy-sinksource.{h,cc}

use std::ffi::c_void as void;

// A Sink is an interfact that consumes a sequence of bytes.
pub trait Sink {
    // Append "bytes[0,n-1]" to this.
    // NOTE: C++ uses const char*
    fn append(&mut self, bytes: *const u8, n: usize);

    // Returns a writable buffer of the specified length for appending. May return a pointer to
    // the caller-owned scratch buffer which must have at least the indicated length. The returned
    // buffer is only valid until the next operation on this sink.
    //
    // After writing at most "length" bytes, call append() with the pointer returned from this
    // function and the number of bytes written. Many append() implementations will avoid copying
    // bytes if this function returned an internal buffer.
    //
    // If a non-scratch buffer is returned, the caller may only pass a prefix of it to append().
    // That is, it is not correct to pass an interior pointer of the returned array to append().
    //
    // The default implementation always returns the scratch buffer.
    #[allow(unused_variables)]
    fn get_append_buffer(&mut self, length: usize, scratch: *mut u8) -> *mut u8 {
        scratch
    }

    // For higher performance, Sink implementations can provide custom append_and_take_ownership()
    // and get_append_buffer_variable() methods. These methods can reduce the number of copies
    // done during compression/decompression.

    // Append "bytes[0,n-1]" to the sink. Takes ownership of "bytes" and calls the deleter
    // function as deleter(deleter_arg, bytes, n) to free the buffer. deleter function must be non
    // NULL.
    //
    // The default implementation just calls append and frees "bytes". Other implementations may
    // avoid a copy while appending the buffer.
    fn append_and_take_ownership(
        &mut self,
        bytes: *mut u8,
        n: usize,
        deleter: fn(*mut void, *const u8, usize),
        deleter_arg: *mut void,
    ) {
        self.append(bytes, n);
        deleter(deleter_arg, bytes, n);
    }

    // Returns a writable buffer for appending and writes the buffer's capacity to
    // "allocated_size". Guarantees "allocated_size" >= "min_size".
    // May return a pointer to the caller-owned scratch buffer which must have "scratch_size" >=
    // "min_size".
    //
    // The returned buffer is only valid until the next operation on this ByteSink.
    //
    // After writing at most "allocated_size" bytes, call append() with the pointer returned from
    // this function and the number of bytes written. Many append() implementations will avoid
    // copy bytes if this function returned an internal buffer.
    //
    // If the sink implementation allocates or reallocates an internal buffer, it should use the
    // "desired_size_hint" if appropriate. If a caller cannot provide a reasonable guess at the
    // desired capacity, it should set "desired_size_hint" = 0.
    //
    // If a non-scratch buffer is returned, the caller may only pass a prefix to it to append().
    // That is, it is not correct to pass an interior pointer to append().
    //
    // The default implementation always returns the scratch buffer.
    #[allow(unused_variables)]
    fn get_append_buffer_variable(
        &mut self,
        min_size: usize,
        desired_size_hint: usize,
        scratch: *mut u8,
        scratch_size: usize,
        allocated_size: &mut usize, // NOTE: C++ uses *size_t
    ) -> *mut u8 {
        *allocated_size = scratch_size;
        scratch
    }
}

pub trait Source {
    // Return the number of bytes left to read from the source.
    fn available(&self) -> usize;

    // Peek at the next flat region of the source. Does not reposition the source. The returned
    // retion is empty iff available()==0.
    //
    // Returns a pointer to the beginning of the region and store is length in "len".
    //
    // The returned region is valid until the next call to skip() or until this object is
    // destroyed, whichever occurs first.
    //
    // The returned region may be larger than available() (for example if this ByteSource is a
    // view on a substring of larger source). The caller is responsible for ensuring that it only
    // reads available() bytes.
    fn peek(&mut self, len: usize) -> *const u8;

    // Skip the next n bytes. Invalidates any buffer returned by a previous call to peek().
    // REQUIRES: available() >= n.
    fn skip(&mut self, n: usize);
}

// A source implementation that yields the contents of a flat array.
pub struct ByteArraySource {
    ptr: *const u8,
    left: usize,
}

impl ByteArraySource {
    pub fn new(ptr: _, left: usize) -> Self {
        Self { ptr, left }
    }
}

impl Source for ByteArraySource {
    fn available(&self) -> usize {
        self.left
    }

    fn peek(&mut self, len: &mut usize) -> *const u8 {
        *len = self.left;
        self.ptr
    }

    fn skip(&mut self, n: usize) {
        self.left -= n;
        unsafe { self.ptr += n };
    }
}

// A snk implementation that writes to a flat array without any bound checks.
pub struct UncheckedByteArraySink {
    dest: *mut u8,
}

impl UncheckedByteArraySink {
    pub fn new(dest: _) -> Self {
        Self { dest }
    }

    // Return the current output pointer so that a caller can see how many bytes were produced.
    // Note: this is not a sink method.
    pub fn current_destination(&self) -> *mut u8 {
        self.dest
    }
}

impl Sink for UncheckedByteArraySink {
    fn append(&mut self, data: *const u8, n: usize) {
        // Do no copying if the caller filled in the result of get_append_buffer()
        unsafe {
            if data != self.dest {
                self.dest.copy_from(data, n);
            }
            self.dest += n;
        }
    }

    fn get_append_buffer(&mut self, _len: usize, scratch: *mut u8) -> *mut u8 {
        self.dest
    }

    fn append_and_take_ownership(
        &mut self,
        bytes: *mut u8,
        n: usize,
        deleter: fn(*mut void, *const u8, usize),
        deleter_arg: *mut void,
    ) {
        unsafe {
            if bytes != self.dest {
                self.dest.copy_from(bytes, n);
                deleter(deleter_arg, bytes, n);
            }
            self.dest += n;
        }
    }

    fn get_append_buffer_variable(
        &mut self,
        _: usize,
        desired_size_hint: usize,
        _: *mut u8,
        _: usize,
        allocated_size: &mut usize,
    ) -> *mut u8 {
        *allocated_suze = desired_size_hint;
        self.dest
    }
}
