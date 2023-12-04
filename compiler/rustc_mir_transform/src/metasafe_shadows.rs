use std::arch::x86_64::__cpuid;

use rustc_borrowck::borrow_set::BorrowSet;
use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_middle::{ty::{TyCtxt, self}, mir::{Body, BorrowKind, ProjectionElem, PlaceElem, Place, Statement, Terminator, visit::{Visitor, PlaceContext}}};
use rustc_mir_dataflow::move_paths::MoveData;

enum LocalUse {
    Statement(Statement),
    Terminator(Terminator)
}

pub struct AddMetaSafeShadows<'tcx> {
    tcx: TyCtxt<'tcx>,
    shadows: FxHashSet<Local>,
    local_uses: FxHashMap<Local, FxHashSet<(Location, PlaceContext)>>
}

impl<'tcx> AddMetaSafeShadows<'tcx> {
    fn walk_projection(&self, place: Place<'tcx>) -> bool {
        for (_,projection) in place.iter_projections() {
            match projection {
                ProjectionElem::Field(_, ty) => {
                    match ty.kind() {
                        ty::RawPtr(_) | ty::Ref(_, _, _) => {
                            return false;
                        }
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        true
    }
}

impl<'tcx> Visitor<'tcx> for AddMetaSafeShadows<'tcx> {
    fn visit_body(&mut self,body: &Body<'tcx>,) {
        if let Some(def) = body.source.def_id().as_local() {
            let param_env = tcx.param_env(def);
            if let Ok(move_data) = MoveData::gather_moves(body, tcx, param_env) {
                let locals_are_invalidated_at_exit = tcx.hir().body_owner_kind(def).is_fn_or_closure();
                let borrow_set = BorrowSet::build(tcx, body, locals_are_invalidated_at_exit, &move_data);
                for (local, decl) in body.local_decls.iter_enumerated() {
                    let local_ty = decl.ty;
                    //the local itself contains a smart pointer
                    //However' if struct contains smart pointers and other structs that contain smart pointers, 
                    //Our initial analysis considers it as a smart pointer.
                    //We probably should change that. A pure smart pointer is fine, but because of projections, things can become wary.
                    if tcx.contains_smart_pointer(local_ty) {
                        if let Some(borrows) = borrow_set.local_map.get(&local) {
                            for index in borrows {
                                let borrow_data = &borrow_set[*Index];
                                let dest_ty = borrow_data.assigned_place.ty(&body.local_decls, tcx).ty;
                                if !tcx.is_smart_pointer(dest_ty) && borrow_data.kind.mutability().is_mut(){
                                    if walk_projection(tcx, borrow_data.borrowed_place, body) {
                                        self.shadows.insert(borrow_data.assigned_place);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn visit_local(&mut self,_local:rustc_middle::mir::Local,_context:rustc_middle::mir::visit::PlaceContext,_location:rustc_middle::mir::Location,) {
      self.local_uses.entry(_local).and_modify(|set|{
        
      })  
    }
}