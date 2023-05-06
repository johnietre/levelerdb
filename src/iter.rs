// TODO: Files?

use crate::{slice::Slice, status::Status};
use std::ffi::c_void;
use std::ptr::null_mut;

pub type CleanupFunction = Box<dyn Fn(*mut c_void, *mut c_void)>;

// NOTE: C++ calls it "Iterator"
pub trait Iter {
    // NOTE: C++ has this as a private member (not a method); implementors should return a
    // reference to their own CleanupNode
    fn cleanup_head<'a>(&'a mut self) -> &'a mut CleanupNode;
    // An iterator is either positioned at a key/value pair, or not valid. This method returns
    // true iff the iterator is valid.
    fn valid(&self) -> bool;
    // Position at the first key in the source. The iterator is valid() after this call iff the
    // source is not empty.
    fn seek_to_first(&mut self);
    // Position at the last key in the source. The iterator is valid() after this call iff the
    // source is not empty.
    fn seek_to_last(&mut self);
    // Position at the first key in the source that is at or past target. The iterator is valid()
    // after this call iff the source contains an entry that comes at or past target.
    fn seek(&mut self, target: &Slice);
    // Moves to the next entry in the source. After this call, valid() is true iff the iterator
    // was not positioned at the last entry in the source.
    // REQUIRES: valid()
    fn next(&mut self);
    // Moves to the previous entry in the source. After this call, valid() is true iff the
    // iterator was not positioned at the first entry in the source.
    // REQUIRES: valid()
    fn prev(&mut self);
    // Return the key for the current entry. The underlying storage for the returned slice is
    // valid only until the next modification of the iterator.
    // REQUIRES: valid()
    fn key(&self) -> Slice;
    // Return the value for the current entry. The underlying storage for the returned slice is
    // valid only until the next modification of the iterator.
    // REQUIRES: valid()
    fn value(&self) -> Slice;
    // If an error has occurred, return it. Else return an ok status.
    fn status(&self) -> Status;
    // Clients are allowed to register function/arg1/arg2 triples that will be invoked when this
    // iterator is destroyed.
    //
    // Note that unlike all of the preceding methods, this method is not abstract and threfore
    // clients should not override it.
    fn register_cleanup(&mut self, function: CleanupFunction, arg1: *mut c_void, arg2: *mut c_void);
    // The default function to be run on drop
    fn drop_func(&mut self) {
        let mut cleanup_head = self.cleanup_head();
        if !cleanup_head.is_empty() {
            cleanup_head.run();
            let mut node = cleanup_head.next.take();
            while let Some(mut n) = node {
                n.run();
                node = n.next.take();
            }
        }
    }
}

// Cleanup functions are stored in a single-linked list.
// This list's head node is inlined in the iterator.
pub struct CleanupNode {
    // The head node is used if the function pointer is not None (null).
    function: Option<CleanupFunction>,
    arg1: *mut c_void,
    arg2: *mut c_void,
    next: Option<Box<CleanupNode>>,
}

impl CleanupNode {
    // True if the node is not used. Only head nodes might be usused.
    fn is_empty(&self) -> bool {
        self.function.is_some()
    }

    // Invokes the cleanup function.
    fn run(&mut self) {
        if let Some(function) = self.function.as_ref() {
            function(self.arg1, self.arg2);
        } else {
            panic!("no cleanup function");
        }
    }
}

impl Default for CleanupNode {
    fn default() -> Self {
        Self {
            function: None,
            arg1: null_mut(),
            arg2: null_mut(),
            next: None,
        }
    }
}

pub struct EmptyIterator {
    status: Status,
    cleanup_head: CleanupNode,
}

impl Iter for EmptyIterator {
    fn cleanup_head(&mut self) -> &mut CleanupNode {
        &mut self.cleanup_head
    }

    fn valid(&self) -> bool {
        false
    }

    fn seek(&mut self, _target: &Slice) {}

    fn seek_to_first(&mut self) {}

    fn seek_to_last(&mut self) {}

    fn next(&mut self) {
        unimplemented!()
    }

    fn prev(&mut self) {
        unimplemented!()
    }

    fn key(&self) -> Slice {
        unimplemented!()
    }

    fn value(&self) -> Slice {
        unimplemented!()
    }

    fn status(&self) -> Status {
        self.status.clone()
    }

    // TODO: Change?
    fn register_cleanup(
        &mut self,
        function: CleanupFunction,
        arg1: *mut c_void,
        arg2: *mut c_void,
    ) {
        self.cleanup_head = CleanupNode {
            function: Some(function),
            arg1,
            arg2,
            next: None,
        };
    }
}

fn new_empty_iterator() -> Box<dyn Iter> {
    Box::new(EmptyIterator {
        status: Status::OK(),
        cleanup_head: Default::default(),
    })
}

fn new_error_iter(status: Status) -> Box<dyn Iter> {
    Box::new(EmptyIterator {
        status,
        cleanup_head: Default::default(),
    })
}
