// leveldb/include/leveldb/env.h

// TODO: Check types of the methods
// TODO: Change all *File to impls in functino args

use crate::{slice::Slice, status::Status};
use std::ffi::c_void;

// TODO: Following block is in C++
// #if defined(_WIN32)
// #if defined(DeleteFile)
// #undef DeleteFile
// #define LEVELDB_DELETEFILE_UNDEFINED
// #endif
// #endif

struct FileLock();
struct RandomAccessFile();
struct WritableFile();

pub trait Env {
    // Create an object that sequentially reads the file with the specified name.
    // On success, stores a pointer to the new file in *result TODO(check source comment) and
    // returns OK.
    // On failure, stores 'None' in *result and returns non-OK. If the file doesn't exist, returns
    // a non-OK status. Implementations should return a NotFound status when the file doesn't
    // exist.
    //
    // The returned file will only be accessed by one thread at a time.
    // TODO: Check type of 'result'
    fn new_sequential_file(
        &mut self,
        fname: &str,
        result: *mut *mut dyn SequentialFile,
    ) -> Status;

    // Create an object supporting random-access from the file with the specified name. On
    // success, stores a pointer to the new file in *result and returns OK. On failure, stores
    // 'None' in *result and returns non-OK. If the file doesn't exist, returns a non-OK status.
    // Implementations should return a NotFound status when the file doesn't exist.
    //
    // The returned file may be concurrently accessed by multiple threads.
    fn new_random_access_file(
        &mut self,
        fname: &str,
        result: *mut *mut RandomAccessFile,
    ) -> Status;

    // Create an object that writes to a new file with the specified name. Deletes any existing
    // file with the same name and creates a new file. On success, stores a pointer to the new
    // file in *result and returns OK. On failure, stores 'None' in *result and returns non-OK.
    //
    // The returned file will only be accessed by one thread at a time.
    fn new_writable_file(
        &mut self,
        result: *mut *mut WritableFile,
    ) -> Status;

    // Create an object that either appends to an existing file, or writes to a file (if the file
    // doesn't exist to begine with). On success, stores a pointer to the new file in *result and
    // returns OK. On failure, stores 'None' in *result and returns non-OK.
    //
    // The returned file will only be accessed by one thread at a time.
    //
    // May return an IsNotSupportedError error if this Env doesn't allow appending to an existing
    // file. Users of Env (including the leveldb implementation) must be prepared to deal with an
    // Env that doesn't support appending.
    fn new_appendable_file(
        &mut self,
        result: *mut *mut WritableFile,
    ) -> Status {
        unimplemented!()
    }

    // Returns true iff the named file exists.
    fn file_exists(&mut self, fname: &str) -> bool;

    // Store in *result the names of the children of the specified dicrectory.
    // The names are relative to "dir".
    // The original contents of *results are dropped.
    // TODO: Check param types
    fn get_children(&mut self, dir: &str, result: &mut Vec<String>) -> Status;

    // Delete the named file.
    //
    // NOTE: The comments below are for deprecated methods in C++
    // The default implementation calls DeleteFile, to support legacy Env implementations. Updated
    // Env implementations must override RemoveFile and ignore the existence of DeleteFile.
    // Updated code calling into the Env API must call remove file instead of DeleteFile.
    //
    // A future release will remove DeleteDir and the default implementation of RemoveDir.
    fn remove_file(&mut self, fname: &str) -> Status {
        unimplemented!()
    }

    // NOTE: C++ has deprecated 'DeleteFile' method

    // Create the specified directory
    fn create_dir(&mut self, dirname: &str) -> Status;

    // Delete the specified directory.
    //
    // NOTE: C++ has deprecation comments similar to those for the 'remove_file' method
    fn remove_dir(&mut self, dirname: &str) -> Status {
        unimplemented!()
    }

    // NOTE: C++ has deprecated 'DeleteDir' method

    // Store the size of fname in *file_size.
    fn get_file_size(&mut self, fname: &str, file_size: &mut u64) -> Status {
        unimplemented!()
    }

    // Rename file src to target.
    fn rename_file(&mut self, src: &str, target: &str) -> Status;

    // Lock the specified file. Used to prevent concurrent access to the same db by multiple
    // processes. On failure, stores 'None' in *lock and returns non-OK.
    //
    // On success, stores a pointer to the object that represents the acquired lock in *lock and
    // returns OK. The caller should call unlock_file(*lock) to release the lock. If the process
    // exits, the lock will be automatically released.
    //
    // If somebody else already holds the lock, finished immediately with a failure. I.e., this
    // call doesn't wait for existing locks to go away.
    //
    // May create the named file if it doesn't alreay exist.
    // TODO: Check types
    fn lock_file(&mut self, fname: &str, lock: &mut &mut FileLock);

    // Release the lock acquired by a previous successful call to lock_file.
    // REQUIRES: lock was returned by a successful lock_file() call.
    // REQUIRES: lock hasn't already be unlocked.
    fn unlock_file(&mut self, lock: &mut FileLock);

    // Arrange to run "function(arg)" once in a background thread.
    //
    // 'function' may run in an unspecified thread. Multiple functions added to the same Env may
    // run concurrently in different threads. I.e., the caller may not assume that background work
    // items are serialized.
    fn schedule(&mut self, function: Box<dyn Fn(*mut c_void)>, arg: *mut c_void);

    // Start a new thread, invoking 'function(arg)' within the new thread. When 'function(arg)'
    // returns, the thread will be destroyed.
    fn start_thread(&mut self, function: Box<dyn Fn(*mut c_void)>, arg: *mut c_void);

    // *path is set to a temporary directory that can be used for testing. It may or may not have
    // just been created. The directory may or may not differ between runs of the same process,
    // but subsequent calls will return the same directory.
    fn get_test_directory(&mut self, path: &mut String) -> Status;

    // Create and return a new log file for storing informational messages.
    fn new_logger(&mut self, fname: &str, result: &mut &mut Logger);

    // Returns the number of micro-seconds since some fixed point in time. Only useful for
    // computing deltas of time.
    fn now_micros(&mut self) -> u64;

    // Sleep/delay the thread for the prescribed number of micro-seconds.
    fn sleep_for_microseconds(micros: i32);
}

// A file abstraction for reading sequentially through a file
pub trait SequentialFile {
    // Read
}

// TODO: See C++ implementation for details
pub struct EnvWrapper {
    target: Box<dyn Env>,
}

impl EnvWrapper {
    pub fn new(t: impl Env) -> Self { Self { target: Box::new(t) } }

    // Return the target to which this env forwards all calls.
    // TODO: Figure out what to return
    pub fn target(&self) -> &Env { return &*self.target }
}

impl Env for EnvWrapper {
    //fn
}

// Return a default environment suitable for the current operating system. Sophisticated users
// may wish to provide their own Env implementation instead of relying on this default
// environment.
//
// The result of Default() belongs to leveldb and must never be deleted.
// NOTE: C++ has this as a static method in Env but since this doesn't work with Rust traits, it's
// implemented as a separate function
//pub fn default() -> Box<dyn Env>; // TODO

// TODO: Possibly use Default trait and different impl based on cfg targets

pub trait Logger {
    // TODO
}
