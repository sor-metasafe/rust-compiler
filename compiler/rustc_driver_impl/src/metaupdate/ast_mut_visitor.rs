use rustc_ast::{mut_visit::MutVisitor, ptr::P, Item};
use smallvec::SmallVec;

use super::MetaUpdateCallbacks;

impl MutVisitor for MetaUpdateCallbacks {
    /// Flat map item visitor in this case for visiting and ast Item
    fn flat_map_item(&mut self, node: P<Item>) -> SmallVec<[P<Item>, 1] {
        SmallVec::new()
    }
}