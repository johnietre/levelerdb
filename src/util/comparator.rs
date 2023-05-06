// leveldb/include/leveldb/comparator.h
// leveldb/util/comparator.cc

use crate::slice::Slice;
#[allow(unused_imports)]
// NOTE: Unknown why this C++ imports this
use crate::util::logging;
#[allow(unused_imports)]
// NOTE: Included solely since C++ includes it; see Note #1 in README.md
use crate::util::no_destructor;

// A Comparator object provides a total order across slices that are used as keys in an sstable or
// a database. A Comparator implementation must be thread-safe since levelerdb may invode its
// methods concurrently from multiple threads.
// TODO: Remove 'self' param from methods?
pub trait Comparator {
    // Three-way comparison. Returns value:
    //   < 0 iff 'a' < 'b', ("iff" means if and only if)
    //   == 0 iff 'a' == 'b',
    //   > 0 iff 'a' > 'b',
    fn compare(&self, a: &Slice, b: &Slice) -> i32;

    // The name of the comparator. Used to check for comparator mismatches (i.e., a DB created
    // with one comparator is accessed using a different comparator).
    //
    // The client of this package should switch to a new name whenever the comparator
    // implementation changes in a way that will cause the relative ordering of any two keys to
    // change.
    //
    // Names starting with "levelerdb." are reserved and shouldn't be used by any clients of this
    // package.
    fn name(&self) -> &'static str;

    // Advanced functions: these are used to reduce the space requirements for internal data
    // structures like index blocks.

    // If 'start' < 'limit', changes 'start' to a short string in ['start','limit'). Simple
    // comparator implementations may return with 'start' unchanged, i.e., an implementation of
    // this method that does nothing is correct.
    fn find_shortest_separator(&self, start: &mut String, limit: &Slice);

    // Changes 'key' to a short string >= 'key'.
    // Simple comparator implementations may return with 'key' unchanged, i.e., an implementation
    // of this method that does nothing is correct.
    fn find_short_successor(&self, start: &mut String);
}

// Return a builtin comparator that uses lexicographic byte-wise ordering. The result repains the
// property of this module and must not be deleted.
pub fn bytewise_comparator() -> &'static dyn Comparator {
    // NOTE: C++ uses NoDestructor here but it's functionality should be unneeded
    static BW_COMPARATOR: BytewiseComparatorImpl = BytewiseComparatorImpl();
    &BW_COMPARATOR
}

struct BytewiseComparatorImpl();

impl Comparator for BytewiseComparatorImpl {
    fn compare(&self, a: &Slice, b: &Slice) -> i32 {
        a.compare(b)
    }

    fn name(&self) -> &'static str {
        "levelerdb.BytewiseComparator"
    }

    fn find_shortest_separator(&self, start: &mut String, limit: &Slice) {
        // Find length of common prefix
        let min_length = start.len().min(limit.size());
        let mut diff_index = 0;
        let start_bytes = unsafe { start.as_mut_vec() };
        while diff_index < min_length && start_bytes[diff_index] == limit[diff_index] {
            diff_index += 1;
        }

        if diff_index >= min_length {
            // Do not shorted in one string is a prefix of the other
        } else {
            let diff_byte = start_bytes[diff_index];
            if diff_byte < 0xff && diff_byte + 1 < limit[diff_index] {
                start_bytes[diff_index] += 1;
                start_bytes.resize(diff_index + 1, 0);
                assert!(self.compare(&start.as_str().into(), limit) < 0);
            }
        }
    }

    fn find_short_successor(&self, key: &mut String) {
        // Find first character than can be incremented
        let n = key.len();
        let key_bytes = unsafe { key.as_mut_vec() };
        for i in 0..n {
            let byte = key_bytes[i];
            if byte != 0xff {
                key_bytes[i] = byte + 1;
                key_bytes.resize(i + 1, 0);
                return;
            }
        }
        // 'key' is a run of 0xffs. Leave it alone.
    }
}
