use rustc_hir::intravisit::Visitor;

use super::MetaUpdateCallbacks;

impl Visitor for MetaUpdateCallbacks {
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
}