//! MetaSafe: Adds validator calls to smart pointer entries.
//! We have to be careful here. Inserting calls right inside smart pointer functions
//! means if sever smart pointer functions call each other, then the validator is run
//! several times. We might as well just run the validator just after a smart pointer routine,
//! and for some functions, we may run it just before, but what should be the criteria in that case?
//! For now, even smart pointer routine that takes a mutable reference (implying it possibly modifies
//! metadata) is considered.

use std::any::Any;
use rustc_middle::mir::Body;
use rustc_middle::query::Key;
use crate::MirPass;
use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_session::Session;

pub struct AddMetaSafeValidatorCalls;

impl<'tcx> MirPass for AddMetaSafeValidatorCalls {
    fn is_enabled(&self, sess: &Session) -> bool {
        sess.opts.unstable_opts.metaupdate
    }

    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        let body_id = body.source.def_id();
        if let Some(impl_id) = tcx.impl_of_method(body_id) {
            let ty = tcx.type_of(impl_id).instantiate_identity();
            if let Some(adt_def) = ty.ty_adt_id() {
                if let Some(validator) = tcx.calculate_validator(adt_def) {

                }
            }
        }
    }
}