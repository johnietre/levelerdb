// leveldb/util/mutexlock.h
// NOTE: See Note #2 in README.md

#![allow(dead_code)]

// Helper struct that locks a mutex on construction and unlocks the mutex when the destructor is
// called (i.e., it's dropped)
// NOTE: This is unneeded due to how Rust handles mutexes
pub struct MutexLock();

impl MutexLock {}
