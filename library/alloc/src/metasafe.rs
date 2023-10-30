/// An implementation for MetaSafe's smart pointer protection trait
/// MetaSafe protects smart pointer metadata by storing it in a separate compartment.
/// We define the MetaUpdate trait which must be implemented by smart pointers

/// The MetaUpdate trait. This must be implemented by all smart pointers
/// A smart pointer with metadata should define how its metadata should be synchronized
/// to prevent attacks
pub trait MetaUpdate {
    /// The synchronize function
    /// It panics if the synchronization fails
    fn synchronize(&self){
        
    }

    /// Enables write protection to the metadata region
    /// By default this uses MPK, but can also use other means
    /// depending on the architecture
    fn enable_metadata_write(&self){

    }

    /// Disables write protection to the metadata region
    fn disable_metadata_write(&self){

    }
}