//! Inserts MPK gates for MetaSafe. The gates are inserted when we call into smart pointer API
//! We do some analysis:
//! A smart pointer API function requires gates if:
//!     It writes to the smart pointer fields (altering the memory of the smart pointer)
//!         For this, we need to make sure the function takes a mut ref or returns a new smart pointer (consider functions like clone)
//!     Calls another function that writes to smart pointer memory:
//!     In this category, we would also like to think about special functions like mem::transmute or mem::replace
//!         If such functions are called and we determine that in any way they result in writing to smart pointer memory,
//!         then we MUST wrap them with MPK open/close gates.

