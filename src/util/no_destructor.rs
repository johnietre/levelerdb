// leveldb/util/no_destructor.h
// leveldb/util/no_destructor_test.h
// NOTE: See Note #1 in README.md

#![allow(dead_code)]

// Wraps an instance whose destructor is never called.
//
// This is intended for use with function-level static variables.
// NOTE: This is unneeded due to how Rust handles destructors (drop)?
pub struct NoDestructor<T> {
    _m: std::marker::PhantomData<T>,
}

impl<T> NoDestructor<T> {}

// NOTE: No tests
