use rustc_hir::{intravisit::Visitor, ItemKind, ExprKind};
use rustc_middle::{hir::nested_filter::OnlyBodies, ty::{EarlyBinder, self, TypeVisitableExt}};
use rustc_span::source_map::LocalDefId;

use super::MetaUpdateCallbacks;

impl Visitor for MetaUpdateCallbacks {
    type NestedFilter = OnlyBodies;

    /// Nested map visiting
    fn nested_visit_map(&mut self) -> Self::Map {
        self.tcx.hir()
    }

    /// Visit an HIR expression. This should help identify which generic types end up
    /// taking smart pointer generics
    fn visit_expr(&mut self, expr: &'v rustc_hir::Expr<'v>) {
        match expr.kind {
            ExprKind::Struct(_,fields , _ ) => {
                let tc = self.tcx.typeck(expr.hir_id.owner.def_id);
                if let Some(parent_ty) = tc.node_type_opt(expr.hir_id) {
                    if !self.tcx.is_smart_pointer(parent_ty) && self.tcx.contains_smart_pointer(parent_ty) {
                        if let ty::Adt(adt, generics) = parent_ty.kind() {
                            for (index, field) in adt.all_fields().enumerate() {
                                let field_ty = field.ty(self.tcx, &generics);
                                if !self.tcx.is_smart_pointer(field_ty) {
                                    self.add_special_field(adt.did(), index, field_ty);
                                }else {
                                    self.add_smart_field(adt.did(), field.did, index);
                                }
                            }
                        }
                    }
                }
            }

            _ => {}
        }
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
                let def_id  = i.item_id().
            },
            _ => {}
        }
    }


    /// visit GenericParam, perhaps we need to save the generics
    fn visit_generics(&mut self, g: &'v rustc_hir::Generics<'v>) {
        
    }
}