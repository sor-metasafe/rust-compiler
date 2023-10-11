use rustc_ast::{mut_visit::MutVisitor, ptr::P, Item, Expr, Crate, NodeId, VariantData};
use smallvec::SmallVec;

use super::MetaUpdateCallbacks;

impl MutVisitor for MetaUpdateCallbacks {

    /// It makes sense that we need to visit a crate,
    /// here we will define more items? such as new structs or is it enough to simply
    /// replace unsafe fields mixed with smart pointers in a struct with a simple tuple?
    /// We'll see
    fn visit_crate(&mut self, c: &mut Crate) {
        todo!()
    }

    /// Visit an AST item, here we are concerned with structs, enums that may need changing.
    fn flat_map_item(&mut self, i: P<Item>) -> SmallVec<[P<Item>; 1]> {
        todo!()
    }

    /// Visits an AST expression to apply possible modifications
    /// In this case, where an expression involves a field expr?
    fn visit_expr(&mut self, e: &mut P<Expr>) {
        todo!()
    }

    /// Here we visit variant data of a struct/enum/tuple
    fn visit_variant_data(&mut self, vdata: &mut VariantData) {
        todo!()
    }
}