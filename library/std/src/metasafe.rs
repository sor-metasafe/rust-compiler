//! The implementation for the MetaSafe shadow memory

#[cfg(not(bootstrap))]
extern crate libc;
#[cfg(not(bootstrap))]
const SHADOW_ADDR: usize = 0x510000000000;
#[cfg(not(bootstrap))]
const SHADOW_SIZE: usize = 0x20000000000;

#[cfg(not(bootstrap))]
#[lang = "metasafe_shadow_alloc"]
/// Allocate the shadow memory for MetaSafe
pub unsafe fn metasafe_shadow_alloc() {
    let addr = core::ptr::from_exposed_addr_mut(SHADOW_ADDR);
    if libc::mmap(
        addr,
        SHADOW_SIZE,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_PRIVATE | libc::MAP_NORESERVE | libc::MAP_ANON,
        -1,
        0,
    ) == libc::MAP_FAILED
    {
        panic!("MetaSafe: failed to create shadow memory");
    }
}


#[cfg(not(bootstrap))]
//#[lang = "metasafe_extern_stack_run"]
/// Run the given routine on a separate stack.
pub unsafe fn metasafe_extern_stack_run<R, F: FnOnce() -> R>(fun: F) -> R {
    fun()
}
