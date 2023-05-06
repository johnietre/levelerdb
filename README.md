# LevelerDB
- A LevelDB clone written in Rust
- List elements denoted with ' * ' mean they should be reviewed for better understanding
# TODO
- Check to see if null terminated strings should be used
- Match visibility with LEVELDB_EXPORT macros
- env.rs
- table/table_builder.rs
- util/cache.rs
- util/crc32c.rs
- util/bloom.rs (tests)
# Check
- iter.rs (possibly move to table)
# Completed
* filter_policy.rs
- slice.rs
* util/coding.rs
* util/hash.rs
- util/histogram.rs
- util/logging.rs
- util/mutexlock.rs
- util/no_destructor.rs
* util/random.rs
- util/status.rs
# Notes
1. The NoDestructor from leveldb/util/no_destructor.h is unneeded due to how Rust handles destructors (dropping) with static instances
2. The MutexLock from leveldb/util/mutexlock.h is unneeded since it's functionality is to provide scoped mutexes but Rust's mutexes do this by default
