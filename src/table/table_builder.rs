// leveldb/include/leveldb/table_builder.h
// leveldb/table/table_builder.cc

use crate::{
    comparator, env, filter_policy,
    options::{CompressionType, Options},
    slice::Slice,
    status::Status,
    table::{block_builder, filter_block, format},
    util::{coding, crc32c},
};

pub struct TableBuilder {
    rep: Rep,
}

impl TableBuilder {
    // Create a builder that will store the contents of the table it is building in file. It is up
    // to the caller to close the file after calling finish().
    pub fn new(...) -> Self;

    // Change the options used by this builder. Note: only some of the option fields can be
    // changed after construction. If a field is not allowed to change dynamically and its value
    // in the structure passed to the constructor is different from its value in the structure
    // passed to this method, this method will return an error without changing any fields.
    pub fn change_options(&mut self, options: &Options) -> Status;

    // Add key,value to the table being constructed.
    // RQRUIRES: key is after any previously added key according to comparator.
    // REQUIRES: finish(), abandon() have noe been called.
    pub fn add(&mut self, key: &Slice, value: &Slice);

    // Advanced operation: flush any buffered key/value pairs to file. Can be used to ensure that
    // two adjacent entries never live in the same data block. Most clients should not need to use
    // this method.
    // RQRUIRES: finish(), abandon() have not bee called.
    pub fn flush(&mut self);

    // Return non-ok iff some error has been detected.
    pub fn status(&self) -> Status;

    // Finish building the table. Stops using the file passed to the constructor after this
    // function returns.
    // REQUIRES: finish(), abandon() have not been called.
    pub fn finish(&mut self) -> Status;

    // Indicate that the contents of this builder should be abandoned. Stops using the file passed
    // to the constructor after this function returns. If the caller is not going to call
    // finish(), it must call abandon() before destroying this builder.
    // REQUIRES: finish(), abandon() have not been called.

    // Number of calls to add() so far.
    pub fn num_entries(&self) -> u64;

    // Size of the file generated so far. If invoked after a successful finish() call, returns the
    // size of the final generated file.
    pub fn file_size(&self) -> u64;

    fn ok(&self) -> bool {
        self.status().ok()
    }

    // NOTE: C++ uses pointers are used instead of references
    fn write_block(&mut self, block: &mut BlockBuilder, handle: &mut BlockHandle);

    // NOTE: C++ uses pointer instead of mut reference
    fn write_raw_block(data: &Slice, _: CompressionType, handle: &mut BlockHandle);
}

impl Drop for TableBuilder {
    // REQUIRES: Either finish() or abandon() has been called.
    fn drop(&mut self);
}

struct Rep {}
