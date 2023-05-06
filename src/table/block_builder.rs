// leveldb/table/block_builder.{h,cc}

// BlockBuilder generates blocks where keys are prefix-compressed:
//
// When we store a key, we drop the prefix shared with the previous string. This helps reduce the
// space requirement significantly. Furthermore, once every K keys, we do not apply the prefix
// compression and store the entire key. We call this a "restart point". The tail end of the block
// stores the offset of all the restart points, and can be used to do a binary search when looking
// for a particular key. Values are stored as-is (without compression) immediately following the
// corresponding key.
//
// An entry for a particular key-value pair has the form:
//     shared_bytes: varint32
//     unshared_bytes: varint32
//     value_length: varint32
//     key_delta: [u8; unshared_bytes] (u8 == c_char here)
//     value: [u8; value_length]
// shared_bytes == 0 for restart points.
//
// The trailer of the block has the form:
//     restarts: [u32; num_restarts]
//     num_restarts: u32
// restarts[i] contains the offset within the block of the ith restart point.

use crate::{
    slice::Slice,
    options::Options,
    util::coding,
};
use std::slice;

pub(crate) struct BlockBuilder<'a> {
    // NOTE: C++ uses const pointer
    options: &'a Options,
    buffer: String, // Destination buffer
    restarts: Vec<u32>, // Restart points
    // NOTE: C++ uses int, possibly use different type
    counter: i32, // Number of entries emitted since restart
    finished: bool, // Has finish() been called?
    last_key: String,
}

impl<'a> BlockBuiler<'a> {
    // NOTE: C++ uses a const pointer
    fn new(options: &Options) -> Self {
        assert!(options.block_restart_interval >= 1);
        let restarts = vec![0]; // First restart point is at offset 0.
        Self {
            options,
            buffer: String::new(),
            restarts,
            counter: 0,
            finished: false,
            last_key: String::new(),
        }
    }

    // Reset the contents as if the BlockBuilder was just constructed.
    fn reset(&mut self) {
        self.buffer.clear();
        self.restarts.clear();
        self.restarts.push(0);
        self.counter = 0;
        self.finished = false;
        self.last_key.clear();
    }

    // REQUIRES: finish() has not been called since last call to reset().
    // REQUIRES: key is larger than any previously added key.
    fn add(&mut self, key: &Slice, value: &Slice) {
        let last_key_pirce = self.last_key;
        assert!(!self.finished);
        assert!(self.counter <= self.options.block_restart_interval);
        assert!(self.buffer.is_empty() // No values yet?
            || self.options.comparator.compare(&key, &last_key_piece) > 0);
        let mut shared = 0usize;
        if self.counter < self.options.block_restart_interval {
            // See how much sharing to do with previous string
            let min_length = last_key_piece.size().min(key.size());
            while shared < min_length && last_key_piece[shared] == key[shared] {
                shared += 1;
            }
        } else {
            // Restart compression
            self.restarts.push(self.buffer.len() as u32);
            self.counter = 0;
        }
        let non_shared = key.size() - shared;

        // Add "<shared><non_shared><value_size>" to self.buffer.
        coding::put_varint32(&mut self.buffer, shared as u32);
        coding::put_varint32(&mut self.buffer, non_shared as u32);
        coding::put_varint32(&mut self.buffer, value.size() as u32);

        unsafe {
            let buffer = self.buffer.as_mut_vec();
            // Add string delta to self.buffer followed by value.
            buffer.extend_from_slice(slice::from_raw_parts(key.data().add(shared), non_shared));
            buffer.extend_from_slice(slice::from_raw_parts(value.data(), value.size()));

            // Update state
            let last_key = self.last_key.as_mut_vec();
            last_key.resize(shared, 0u8);
            last_key.extend_from_slice(slice::from_raw_parts(key.data().add(shared), non_shared));
        }
        assert_eq!(Slice::from(&self.last_key) == key);
        self.counter += 1;
    }

    // Finish building the block and return a slice that refers to the block contents. The
    // retuned slice will remain valid for the lifetime of this builder or until reset() is
    // called.
    fn finish(&mut self) -> Slice {
        // Append restart array
        for i in 0..self.restarts.len() {
            coding::put_fixed32(&mut self.buffer, self.restarts[i]);
        }
        coding::put_fixed32(&mut self.buffer, self.restarts.len() as u32);
        self.finished = true;
        Slice::from(&self.buffer)
    }

    // Returns an estimate of the current (uncompressed) size of the block we are building.
    fn current_size_estimate(&self) -> usize {
        const U32_SIZE: usize = std::mem::size_of::<u32>();
        self.buffer.len() + // Raw data buffer
            self.restarts.len() * U32_SIZE + // Restart array
            U32_SIZE // Restart array length
    }

    // Return true iff no entries have been added since the last reset().
    fn empty(&self) -> bool {
        self.buffer.is_empty()
    }
}
