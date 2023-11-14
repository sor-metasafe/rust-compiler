use core::ptr;

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

/// MetaSafe: Allocate a shadow for a struct that contains smart pointers
/// This is automatically called when the struct is being constructed.
/// The returned pointer is stored at the tail of the struct for easy access.
//#[lang = "metasafe_shadow_alloc"]
pub unsafe fn metasafe_alloc_shadow(_size: usize) -> *mut u8 {
    // for now it returns NULL
    ptr::null_mut()
}

/// MetaSafe: Deallocate a shadow for a struct that contains smart pointers
/// This is automatically called when the struct is dropped.
/// The pointer can be obtained from the tail/back of the struct.
//#[lang = "metasafe_shadow_free"]
pub unsafe fn metasafe_dealloc_shadow(_ptr: *mut u8) {
    // do nothing for now.
}