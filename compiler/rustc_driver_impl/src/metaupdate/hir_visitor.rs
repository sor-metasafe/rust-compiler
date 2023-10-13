use rustc_hir::{intravisit::Visitor, ItemKind};
use rustc_middle::hir::nested_filter::OnlyBodies;

use super::MetaUpdateCallbacks;

impl Visitor for MetaUpdateCallbacks {
    type NestedFilter = OnlyBodies;

    /// Nested map visiting
    fn nested_visit_map(&mut self) -> Self::Map {
        self.tcx.hir()
    }

    /// Visit an HIR expression. This should help identify which generic types end up
    /// taking smart pointer generics
    fn visit_expr(&mut self, ex: &'v rustc_hir::Expr<'v>) {
        
    }

    /// Visit a field expression. This should help identify which fields need to be handled 
    /// specially ... eg fields which take smart pointers.
    fn visit_expr_field(&mut self, field: &'v rustc_hir::ExprField<'v>) {
        
    }

    /// field definitions are also part of us. Perhaps this is just for bookkeeping.
    /// We'll see.
    fn visit_field_def(&mut self, s: &'v rustc_hir::FieldDef<'v>) {
        
    }

    /// visit ADT definitions, this simply adds them to the map.
    fn visit_item(&mut self, i: &'v rustc_hir::Item<'v>) {
        match i.kind {
            ItemKind::Struct(variant_data, generics) => {
                for param in generics.params {
                    param.d
                }
            },
            _ => {}
        }
    }


    /// visit GenericParam, perhaps we need to save the generics
    fn visit_generics(&mut self, g: &'v rustc_hir::Generics<'v>) {
        
    }
}