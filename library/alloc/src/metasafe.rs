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