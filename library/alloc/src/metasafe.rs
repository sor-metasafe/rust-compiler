/// An implementation for MetaSafe's smart pointer protection trait
/// MetaSafe protects smart pointer metadata by storing it in a separate compartment.
/// We define the MetaUpdate trait which must be implemented by smart pointers
/// The MetaUpdate trait. This must be implemented by all smart pointers
/// A smart pointer with metadata should define how its metadata should be synchronized
/// to prevent attacks
pub trait MetaUpdate {
    /// The synchronize function
    /// It panics if the synchronization fails
    fn synchronize(&self);
}

/// MetaSafe: Allocate a shadow memory for smart pointers
#[cfg(not(bootstrap))]
extern crate libc;
#[cfg(not(bootstrap))]
const SHADOW_ADDR: usize = 0x510000000000;
#[cfg(not(bootstrap))]
const SHADOW_SIZE: usize = 0x20000000000;
#[cfg(not(bootstrap))]
const SHADOW_MASK: usize = 0x1FFFFFFFFFF;
#[cfg(not(bootstrap))]
#[lang = "metasafe_shadow_alloc"]
pub unsafe fn metasafe_shadow_alloc() {
    use std::os::raw::c_void;

    if libc::mmap(SHADOW_ADDR as *mut c_void, SHADOW_SIZE, libc::PROT_READ|libc::PROT_WRITE, libc::MAP_PRIVATE|libc::MAP_ANON|libc::MAP_NORESERVE, -1, 0) == libc::MAP_FAILED {
        panic!("MetaSafe: Unable to create shadow map");
    }
}

/// MetaSafe: Deallocate a shadow for a struct that contains smart pointers
/// This is automatically called when the struct is dropped.
/// The pointer can be obtained from the tail/back of the struct.
#[cfg(not(bootstrap))]
#[lang = "metasafe_shadow_free"]
pub unsafe fn metasafe_shadow_free() {
    // do nothing for now. => will the shadow memory be unmapped automatically?
}