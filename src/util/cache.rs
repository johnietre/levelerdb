// leveldb/include/leveldb/cache.h
// leveldb/util/cache.cc

use std::ffi::c_void;
// TODO: port modules?
// TODO: Thread annotations
use crate::slice::Slice;
use crate::util::hash;
// NOTE Only imported since C++ imports it; see Note #2 in README.md
#[allow(unused_imports)]
use crate::util::mutexlock;

// Create a new cache with a fixed size capacity. This implementation of Cache uses a
// least-recently-used eviction policy.
// NOTE: Uses ShardedLRUCache
// NOTE: Box<dyn _> or Box<ShardedLRUCache> or unboxed?
pub fn new_lru_cache(capacity: usize) -> Box<dyn Cache> {
    Box::new(ShardedLRUCache::with_capacity(capacity))
}

// Opaque handle to an entry stored in the cache.
pub trait Cache {
    // NOTE: Virtual deconstructor in C++, just Drop here?
    // Destroys all existing entries by calling the "deleter" function that was passed to the
    // constructor

    // Insert a mapping from key.value into the cache and assign it the specified charge against
    // the total capacity.
    //
    // Returns a handle that corresponds to the mapping. The caller must call release(handle) when
    // the returned mapping is no longer needed.
    //
    // When the isnerted entry is no longer needed, the key and value will be passed to "deleter".
    // TODO/NOTE: Virtual function in C++
    // TODO: Use Generic rather than box?
    fn insert(
        &mut self,
        key: &Slice,
        value: *const c_void,
        charge: usize,
        deleter: Box<dyn Fn(&Slice, *const c_void)>,
    ) -> Box<dyn Handle>;

    // If the cache has no mappying for "key", returns None.
    //
    // Else return a handle that corresponds to the mapping. The caller must call
    // self.release(&mut self, handle) when the returned mapping is no longer needed.
    // NOTE: Returns nullptr in C++ rather than None
    // TODO
    fn lookup(&mut self, key: &Slice) -> Option<Box<dyn Handle>>;

    // Release a mapping returned by a previous lookup(&mut self, ).
    // REQUIRES: handle must not have been released yet.
    // REQUIRES: handle must have been returned by a method on self.
    // TODO: Use generic box?
    fn release(&mut self, handle: Box<dyn Handle>);

    // Return the value encapsulated in a handle returned by a successful lookup(&mut self, ).
    // REQUIRES: handle must not have been released yet.
    // REQUIRES: handle must have been returned by a method on self.
    fn value(&mut self, handle: Box<dyn Handle>) -> *const c_void;

    // If the cache contains entry for key, erase it. Note that the underlying entry will be kept
    // around until all existing handles to it have been released.
    fn erase(&mut self, slice: &Slice);

    // Return a new numeric id. May be used by multiple clients who are sharing the same cache to
    // partition to key space. Typically the client will allocate a new id at startup and prepend
    // the id to its cache keys.
    fn new_id(&mut self) -> u64;

    // Remove all cache entries that aren't actively in use. Memory-constrained applications may
    // wish to call this method to reduce memory usage.
    // Default implementation of prune() does nothing. Trait implementors are strongly encouraged
    // to override the default implementation. A future release of leveldb my change prune() to a
    // pure abstract method.
    fn prune(&mut self) {}

    // Return an estimate of the combined charges of all elements stored in the cache.
    fn total_charge(&self) -> usize;
}

// TODO: Comments
struct LRUHandle {
    value: *mut c_void,
    deleter: fn(&Slice, *mut c_void),
    next_hash: *mut LRUHandle,
    next: *mut LRUHandle,
    prev: *mut LRUHandle,
    charge: usize, // TODO(opt): Only allow u32?; NOTE: This is a source code todo
    key_length: usize,
    // Whether entry is in cache.
    in_cache: bool,
    // References, including cache reference, if present.
    refs: u32,
    // Hash of key(); used for fast sharding and comparisons
    hash: u32,
    // Beginning of key
    // NOTE: Char
    key_data: *mut u8,
}

impl LRUHandle {
    pub fn key(&self) -> Slice {
        // 'next' is only equal to this if the LRU handle is the list head of an empty list. List
        // heads never have meaningful keys.
        assert_ne!(self.next, self as *mut Self);
        Slice::from_ptr(self.key_data, self.key_length)
    }

    // NOTE: Done so MaybeUninit doesn't have to be used
    fn _new() -> Self {
        Self {
            value: null_mut(),
            deleter: |_, _| {},
            next_hash: null_mut(),
            next: null_mut(),
            prev: null_mut(),
            charge: 0,
            key_length: 0,
            in_cache: false,
            refs: 0,
            hash: 0,
            key_data: null_mut(),
        }
    }
}

// NOTE: "We" meaning leveldb writers
// We provide our own simple hash table since it removes a whole bunch of porting hacks and is
// also faster than some of the built-in hash table implementations in some of the
// compiler/runtime combos tested. E.g., readrandom speeds up by ~5% over the g++ 4.4.3's builtin
// hashtable
struct HandleTable {
    // The table consists of an array of buckets where each bucket is a linked list of cache
    // entries that hash into the bucket
    length: u32,
    elems: u32,
    // NOTE: Could use NonNull
    list: *mut *mut LRUHandle,
}

impl HandleTable {
    fn new() -> Self {
        Self {
            length: 0,
            elems: 0,
            // NOTE: If NonNull, use NonNull::dangling
            list: null_mut(),
        }
    }

    pub fn lookup(&mut self, key: &Slice, hash: u32) -> *mut LRUHandle {
        self.find_pointer(key, hash);
    }

    pub fn insert(&mut self, h: *mut LRUHandle) -> *mut LRUHandle {
        unsafe {
            let ptr = self.find_pointer((*h).key(), (*h).hash);
            let old = *ptr;
            (*h).next_hash = if old.is_null() {
                null_mut()
            } else {
                (*old).next_hash
            };
            *ptr = h;
            if old.is_null() {
                self.elems += 1;
                if self.elems > self.length {
                    // Since each cace entry is fairly large, we aim for a small average linked
                    // list length (<= 1).
                    self.resize();
                }
            }
            old
        }
    }

    pub fn remove(&mut self, key: &Slice, hash: u32) -> *mut LRUHandle {
        unsafe {
            let ptr = self.find_pointer(key, hash);
            let result = *ptr;
            if !result.is_null() {
                *ptr = (*result).next_hash;
                self.elems -= 1;
            }
            result
        }
    }

    // Return a pointer to slot that points to a cache entry that matches key/hash. If there is no
    // such cache entry, return a pointer to the trailing slot in the corresponding linked list.
    fn find_pointer(&self, key: &Slice, hash: u32) -> *mut *mut LRUHandle {
        unsafe {
            let mut ptr = self.list.add(hash & (self.length - 1) as u32);
            while !(*ptr).is_null() && ((**ptr).hash != hash || key != (**ptr).key()) {
                ptr = (&mut (*ptr).next_hash) as _;
            }
            ptr
        }
    }

    fn resize(&mut self) {
        unsafe {
            let mut new_length = 4;
            while new_length < self.elems {
                new_length *= 2;
            }
            let new_list = alloc::alloc_zeroed(Layout::array::<*mut LRUHandle>(new_length).unwrap())
                as *mut *mut LRUHandle;
            let mut count = 0u32;
            for i in 0..self.length {
                let mut h = *self.list.add(i as usize);
                while !h.is_null() {
                    let next = (*h).next_hash;
                    let hash = (*h).hash;
                    let ptr = &(*new_list.add(hash as usize & (new_length - 1))) as *mut _;
                    (*h).next_hash = *ptr;
                    h = next;
                    count += 1;
                }
            }
            assert_eq!(self.elems, count);
            alloc::dealloc(
                self.list.cast(),
                Layout::array::<*mut LRUHandle>(self.length),
            );
            self.list = new_list;
            self.length = new_length;
        }
    }
}

impl Drop for HandleTable {
    fn drop(&mut self) {
        unsafe {
            alloc::dealloc(
                self.list.cast(),
                Layout::array::<*mut LRUHandle>(self.length),
            );
        }
    }
}

struct LRUCacheGuardedData {
    usage: usize,
    // Dummy head of LRU list.
    // lru.prev is newest entry, lru.next is oldest entry.
    // Entries have refs==1 and in_cache==true.
    lru: LRUHandle,

    // Dummy head of in-use list.
    // Entries are in use by clientts, and have refs >= 2 and in_cache==true.
    in_use: LRUHandle,

    table: HandleTable,
}

// A single shard of sharded cache
struct LRUCache {
    // Initialized before use.
    capacity: usize,

    // mutex protects the following state.
    // NOTE: C++ mutex doesn't own data like Rust and synchronous usage requires a data be owned
    // by mutex (or other) in Rust.
    mutex: port::Mutex<LRUCacheGuardedData>,
}

impl LRUCache {
    fn new() -> Self {
        // Make empty circular linked lists.
        let mut lru = LRUHandle::_new();
        lru.next = &mut lru as *mut _;
        lru.prev = &mut lru as *mut _;
        let mut in_use = LRUHandle::_new();
        in_use.next = &mut in_use as *mut _;
        in_use.prev = &mut in_use as *mut _;
        let mut cache = Self {
            capacity: 0,
            mutex: port::Mutex::new(LRUCacheGuardedData {
                usage: 0,
                lru: {
                },
                in_use,
                table: LRUHandle::_new(),
            }),
        };
    }

    // Searate from constructor so caller can easily make an array of LRUCache.
    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity;
    }

    // Like Cache methods, but with an extra "hash" parameter.
    pub fn insert(
        &mut self,
        key: &Slice,
        hash: u32,
        value: *mut c_void,
        charge: usize,
        deleter: fn(&Slice, *mut c_void),
    ) -> *mut Handle; // TODO: Cache::Handle

    pub fn lookup(&mut self, key: &Slice, hash: u32) -> *mut Handle; // TODO: Cache::Handle

    pub fn release(&mut self, handle: *mut Handle); // TODO: Cache::Handle

    pub fn erase(&mut self, key: &Slice, hash: u32);

    pub fn prune(&mut self);

    pub fn total_charge(&self) -> usize {
        *self.mutex.lock().unwrap().usage
    }

    fn lru_remove(&mut self, e: *mut LRUHandle) {
        unsafe {
            (*(*e).next).prev = (*e).prev;
            (*(*e).prev).next = (*e).next;
        }
    }

    fn lru_append(&mut self, list: *mut LRUHandle, e: *mut LRUHandle) {
        unsafe {
            // Make "e" newest entry by inserting just before *list.
            (*e).next = list;
            (*e).prev = (*list).prev;
            (*(*e).prev).next = e;
            (*(*e).next).prev = e;
        }
    }

    fn ref(&mut self, e: *mut LRUHandle) {
        unsafe {
            if (*e).refs == 1 && (*e).in_cache { // If on self.lru list, move to self.in_use list.
                self.lru_remove(e);
                self.lru_append(&mut self.in_use as *mut _, e);
            }
            (*e).refs += 1;
        }
    }

    fn unref(&mut self, e: *mut LRUHandle) {
        unsafe {
            assert!((*e).refs > 0);
            (*e).refs -= 1;
            if (*e).refs == 0 { // Deallocate.
                assert!(!(*e).in_cache);
                (*e).deleter((*e).key(), (*e).value);
                alloc::dealloc(e as *mut _, Layout::new::<LRUHandle>());
            } else if (*e).in_cache && (*e).refs == 1 {
                // No longer in use; move to self.lru list.
                self.lru_remove(e);
                self.lru_append(&mut self.lru as *mut _, e);
            }
        }
    }

    fn finish_erase(&mut self, e: *mut LRUHandle);
}

impl Drop for LRUCache {
    fn drop(&mut self) {
        unsafe {
            // Error if caller has an unlreaded handle
            assert!(self.in_use.next == &self.in_use as *const _);
            let mut e = self.lru.next;
            while e != &self.lru as *const _ {
                let next = (*e).next;
                assert!((*e).in_cache);
                (*e).in_cache = false;
                assert_eq!((*e).refs, 1); // Invariant of self.lru list.
                self.unref(e);
                e = next;
            }
        }
    }
}

const NUM_SHARD_BITS: usize = 4;
const NUM_SHARDS: usize = 1 << NUM_SHARD_BITS;

// TODO: Access?
pub(crate) struct ShardedLRUCache {
    shard: [LRUCache; NUM_SHARDS],
    id_mutex: port::Mutex,
    last_id: u64,
}

impl ShardedLRUCache {
    // NOTE: static func in C++
    // TODO: Watch Hash method
    #[inline]
    fn hash_slice(s: &Slice) -> u32 {
        hash(s.data(), s.size(), 0)
    }

    fn shard(hash: u32) -> u32 {
        hash >> (32 - NUM_SHARD_BITS as u32)
    }
}

impl Cache for ShardedLRUCache {
    fn new_id(&mut self) -> u64 {
        let mut last_id = id_mutex.lock().unwrap();
        // TODO
    }

    fn prune(&mut self) {
        for s in 0..NUM_SHARDS {
            shars[s].prune();
        }
    }

    fn total_charge(&self) -> usize {
        let mut total = 0;
        for s in 0..NUM_SHARDS {
            total += self.shard[s].total_charge();
        }
        total
    }
}
