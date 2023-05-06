// leveldb/table/filter_block.{h,cc}

use crate::{
    coding,
    filter_policy::FilterPolicy,
    slice::Slice,
};
use std::ffi::c_char;
use std::slice;

// TODO: Below
// See doc/table_format.md for an explanation of the filter block format.

// Generate new filter every 2kb of data
const FILTER_BASE_LG: usize = 11;
const FILTER_BASE: usize = 1 << FILTER_BASE_LG;

// A FilterBlockBuilder is used to construt all of the filters for a particular Table. It
// generates a single string which is stored as a special block in the Table.
//
// The sequence of calls to FilterBlockBuilder must match the regexp:
//      (StartBlock AddKey*)* Finish
pub(crate) struct FilterBlockBuilder {
    // NOTE: C++ uses const*, possibly use reference
    // NOTE: doing this as a nasty psuedo-work-around so that policy can be called without unsafe.
    // The C++ code never checks for nullptr so I won't either.
    // TODO: Change drop() method when changing this
    policy: Box<dyn FilterPolicy>,
    keys: String, // Flattened key contents
    start: Vec<usize>, // Starting index in self.keys of each key
    result: String, // Filter data computed so far
    tmp_keys: Vec<Slice>, // policy.create_filter() argument
    filter_offsets: Vec<u32>,
}

impl FilterBlockBuilder {
    // TODO: policy
    pub fn new(policy: &dyn FilterPolicy>) -> Self {
        Self {
            policy: unsafe {
                Box::from_raw(policy as *const dyn FilterPolicy as *mut dyn FilterPolicy)
            },
            keys: String::new(),
            start: Vec::new(),
            result: String::new(),
            tmp_keys: Vec::new(),
            filter_offsets: Vec::new(),
        }
    }

    pub fn start_block(&mut self, block_offset: u64) {
        let filter_index = block_offset / FILTER_BASE as u64;
        assert!(filter_index >= self.filter_offsets.len() as u64);
        while filter_index > self.filter_offsets.len() as u64 {
            self.generate_filter();
        }
    }

    pub fn add_key(&mut self, key: &Slice) {
        let k = key;
        self.start.push(self.keys.len());
        unsafe { self.keys.as_mut_vec().extend_from_slice(slice::from_raw(k.data(), k.size())) };
    }

    pub fn finish(&mut self) -> Slice {
        if self.start.is_empty() {
            self.generate_filter();
        }

        // Append array of per-filter offsets.
        let array_offset = self.result.len() as u32;
        for i in 0..self.filter_offsets.len() {
            let fo = self.filter_offsets[i] as u32;
            coding::put_fixed32(&mut self.result, fo);
        }

        coding::put_fixed32(&mut self.result, array_offset);
        self.result.push(FILTER_BASE_LG); // Save encoding parameter in result
        Slice::from(&self.result)
    }

    fn generate_filter(&mut self) {
        let num_keys = self.start.len();
        if num_keys == 0 {
            // Fast path if there are no keys for this filter
            self.filter_offsets.push(self.result.len());
            return;
        }

        // Make list of keys from flattened key structure
        self.start.push(self.keys.size()); // Simplify length computation
        self.tmp_keys.resize(num_keys, Slice::new());
        for i in 0..num_keys {
            let base = unsafe { self.keys.as_ptr().cast() }
            let length = self.start[i + 1] - self.start[0];
            self.tmp_keys[i] = Slice::from_raw(base, length);
        }

        // Generate filter for current set of keys and append to self.result.
        self.filter_offsets.push(self.result.len());
        let sl = &self.tmp_keys[0] as *const [Slice; 1];
        unsafe { self.policy.create_filter(&*sl, num_keys, &mut self.result) };

        self.tmp_keys.clear();
        self.keys.clear();
        self.start.clear();
    }
}

impl Drop for FilterBlockBuiler {
    fn drop(&mut self) {
        // TODO: Remove when changing self.policy
        std::mem::forget(self.policy);
    }
}

pub(crate) struct FilterBlockReader {
    // NOTE: C++ uses const*, possibly use reference
    policy: Box<dyn FilterPolicy>,
    data: *const c_char, // Pointer to filter data (at block-start)
    offset: *const c_char, // Pointer to beginning of offset array (at block-end)
    num: usize, // Number of entries in offset array
    base_lg: usize // Encoding parameter (see FILTER_BASE_LG)
}

impl FilterBlockReader {
    pub fn new(policy: Box<dyn FilterPolicy>, contents: &Slice) -> Self {
        let mut reader = Self {
            policy,
            data: 0 as *const _,
            offset: 0 as *const _,
            num: 0,
            base_lg: 0,
        };
        let n = contents.size();
        // 1 byte for base_lg and 4 for start of offset array
        if n < 5 {
            return reader;
        }
        reader.base_lg = contents[n - 1];
        unsafe {
            let last_word = coding::decode_fixed32(contents.data().add(n - 5));
            if last_word > n - 5 {
                return reader;
            }
            reader.data = contents.data();
            reader.offset = reader.data.add(last_word as usize);
            reader.num = (n - 5 - last_word) / 4;
        }
        reader
    }

    pub fn key_may_match(&mut self, block_offset: u64, key: &Slice) -> bool {
        let index = block_offset >> self.base_lg as u64;
        if index < self.num as u64 {
            unsafe {
                let start = coding::decode_fixed32(self.offset.add(index as usize * 4)) as usize;
                let limit = coding::decode_fixed32(self.offset.add(index as usize * 4 + 4)) as usize;
                if start <= limit && limit <= self.offset as usize - self.data as usize {
                    return policy.key_may_match(key, Slice::from_raw(self.data.add(start), limit - start);
                } else if start == limit {
                    // Empty filters do not match any keys
                    return false;
                }
            }
        }
        return true; // Errors are treated as potential matches
    }
}

#[cfg(tests)]
mod tests {
    use super::*;
    use crate::{filter_policy::, util::{hash, logging}};

    struct TestHashFilter;

    impl FilterPolicy for TestHashFilter {
        fn name(&self) -> &'static str { "TestHashFilter" }

        fn create_filter(&self, keys: &[Slice], n: usize, dst: &mut String) {
            for i in 0..n {
                coding::put_fixed32(dst, hash::hash(keys[i].data(), keys[i].size(), 1));
            }
        }

        fn key_may_match(&self, key: &Slice, filter: &Slice) -> bool {
            let h = hash::hash(key.data(), key.size(), 1);
            let mut i = 0;
            while i + 4 < filter.size() {
                if h == coding::decode_fixed32(unsafe { filter.data().add(i) }) {
                    return true;
                }
                i += 4;
            }
            false
        }
    }

    const POLICY: TestHashFilter = TestHashFilter;

    #[test]
    fn test_empty_builder() {
        let mut builder = FilterBlockBuilder(&POLICY);
        let block = builder.finish();
        assert_eq!("\\x00\\x00\\x00\\x00\\x0b", logging::escape_string(block));
        let mut reader = FilterBlockReader::new(&POLICY, &block);
        assert!(reader.key_may_match(0, Slice::from("foo")));
        assert!(reader.key_may_match(100_000, Slice::from("foo")));
    }

    #[test]
    fn test_single_chunk() {
        let mut builder = FilterBlockBuilder(&POLICY);
        builder.start_block(100);
        builder.add_key(Slice::from("foo"));
        builder.add_key(Slice::from("bar"));
        builder.add_key(Slice::from("box"));
        builder.start_block(200);
        builder.add_key(Slice::from("box"));
        builder.start_block(300);
        builder.add_key(Slice::from("hello"));
        let block = builder.finish();
        let mut reader = FilterBlockReader::new(&POLICY, &block);
        assert!(reader.key_may_match(100, "foo"));
        assert!(reader.key_may_match(100, "bar"));
        assert!(reader.key_may_match(100, "box"));
        assert!(reader.key_may_match(100, "hello"));
        assert!(reader.key_may_match(100, "foo"));
        assert!(!reader.key_may_match(100, "missing"));
        assert!(!reader.key_may_match(100, "other"));
    }

    #[test]
    fn test_multi_chunk() {
        let mut builder = FilterBlockBuilder(&POLICY);

        // First filter
        builder.start_block(0);
        builder.add_key("foo");
        builder.start_block(2000);
        builder.add_key("bar");

        // Second filter
        builder.start_block(3100);
        builder.add_key("box");

        // Third filter is empty

        // Last filter
        builder.start_block(9000);
        builder.add_key("box");
        builder.add_key("hello");

        let block = builder.finish();
        let mut reader = FilterBlockReader::new(&POLICY, &block);

        // Check first filter
        assert!(reader.key_may_match(0, "foo"));
        assert!(reader.key_may_match(2000, "bar"));
        assert!(!reader.key_may_match(0, "box"));
        assert!(!reader.key_may_match(0, "hello"));

        // Check second filter
        assert!(reader.key_may_match(3100, "box"));
        assert!(!reader.key_may_match(3100, "foo"));
        assert!(!reader.key_may_match(3100, "bar"));
        assert!(!reader.key_may_match(3100, "hello"));

        // Check third filter (empty)
        assert!(!reader.key_may_match(4100, "foo"));
        assert!(!reader.key_may_match(4100, "bar"));
        assert!(!reader.key_may_match(4100, "box"));
        assert!(!reader.key_may_match(4100, "hello"));

        // Check last filter
        assert!(reader.key_may_match(9000, "box"));
        assert!(reader.key_may_match(9000, "hello"));
        assert!(!reader.key_may_match(9000, "foo"));
        assert!(!reader.key_may_match(9000, "bar"));
    }
}
