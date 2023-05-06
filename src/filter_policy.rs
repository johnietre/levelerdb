// leveldb/include/leveldb/filter_policy.h

use crate::slice::Slice;

pub trait FilterPolicy {
    // Return the name of this policy. Note that if the filter encoding changes in an incompatible
    // way, the name returned by this method must be changed. Otherwise, old incompatible filters
    // may be passed to methods of this type.
    fn name(&self) -> &'static str;

    // keys[0,n-1] contains a list of keys (potentially with duplicates) that are ordered
    // according to the user supplied comparator.
    // Append a filter that summarizes keys[0,n-1] to *dst.
    //
    // Warning: don't change the initial contents of *dst. Instead, append the newly constructed
    // filter to *dst.
    // NOTE: C++ takes 'n' as an int
    fn create_filter(&self, keys: &[Slice], n: usize, dst: &mut String);

    // 'filter' contains the data appended by a preceding call to `create_filter` on this trait.
    // This method must return true if the key was in the list of keys passed to `create_filter`.
    // This method may return true or false if the key was not on the list, but it should aim to
    // return false with a high probability.
    fn key_may_match(&self, key: &Slice, filter: &Slice) -> bool;
}
