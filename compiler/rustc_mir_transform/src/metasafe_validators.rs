//! MetaSafe: Adds validator calls to smart pointer entries.
//! We have to be careful here. Inserting calls right inside smart pointer functions
//! means if sever smart pointer functions call each other, then the validator is run
//! several times. We might as well just run the validator just after a smart pointer routine,
//! and for some functions, we may run it just before, but what should be the criteria in that case?
//! For now, even smart pointer routine that takes a mutable reference (implying it possibly modifies
//! metadata) is considered.

use rustc_ast::Mutability;
use rustc_data_structures::fx::FxHashMap;
use rustc_middle::mir::{TerminatorKind, Operand, BasicBlockData, Terminator, SourceInfo, Place};
use rustc_middle::mir::{Body, patch::MirPatch, UnwindAction, CallSource};
use rustc_middle::query::Key;
use crate::MirPass;
use rustc_middle::ty::{self, Ty, TyCtxt, List};
use rustc_session::Session;
use rustc_hir as hir;

pub static SAFE_CRATES: [&'static str; 23] = [
    "std",
    "alloc",
    "backtrace",
    "core",
    "panic_abort",
    "panic_unwind",
    "portable-simd",
    "proc_macro",
    "profiler_builtins",
    "sysroot",
    "rustc",
    "stdarch",
    "rtstartup",
    "rustc_std_workspace_alloc",
    "rustc_std_workspace_core",
    "rustc_std_workspace_std",
    "std_detect",
    "hash_brown",
    "libc",
    "cfg_if",
    "unwind",
    "object",
    "adler",
];

pub struct AddMetaSafeValidatorCalls;

impl<'tcx> MirPass<'tcx> for AddMetaSafeValidatorCalls {
    fn is_enabled(&self, sess: &Session) -> bool {
        sess.opts.unstable_opts.metaupdate
    }

    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        let body_id = body.source.def_id();
        let crate_name = tcx.crate_name(body_id.krate).to_string();
        if SAFE_CRATES.contains(&crate_name.as_str()) {
            return;
        }

        let bbs = &body.basic_blocks;
        let mut validators = FxHashMap::default();
        let mut drop_validators = FxHashMap::default();
        let mut patch = MirPatch::new(body);

        //collect basic blocks where to insert validator calls.
        for idx in bbs.indices() {
            let terminator = bbs[idx].terminator();
            match &terminator.kind {
                TerminatorKind::Call { func,args, destination, target, .. } => {
                    let callee = func.ty(&body.local_decls, tcx);
                    match callee.kind() {
                        ty::FnDef(def_id, _) => {
                            let fn_sig = tcx.fn_sig(def_id).skip_binder();
                            if fn_sig.unsafety() != hir::Unsafety::Unsafe {
                                continue;
                            }
                            
                            if let Some(impl_id) = tcx.impl_of_method(*def_id) {
                                let impl_ty = tcx.type_of(impl_id).instantiate_identity();
                                if let Some(validator) = impl_ty.ty_adt_id().and_then(|id|{
                                    tcx.calculate_validator(id)
                                }) {
                                    let mut arg_iter = args.iter();
                                    if let Some(first_arg) = arg_iter.next() {
                                        let arg_ty = first_arg.ty(&body.local_decls, tcx);
                                        if arg_ty.peel_refs().ty_adt_id() == impl_ty.ty_adt_id() {
                                            match arg_ty.kind() {
                                                ty::Ref(_, _, mutbl) => {
                                                    if let Mutability::Mut = *mutbl {
                                                        validators.insert(idx, (first_arg.clone(),  validator, target.clone()));
                                                    }
                                                },
                                                _ => {}
                                            }
                                            continue;
                                        }

                                    }

                                    // So we don't have any args that match ours. Is it possible we are creating a new type of us?
                                    // If so, then we need to perform a validation for the returned type.
                                    let dest_ty = destination.ty(&body.local_decls, tcx).ty;
                                    if dest_ty.peel_refs().ty_adt_id() == impl_ty.ty_adt_id() {
                                        let operand = Operand::Copy(*destination);
                                        validators.insert(idx, (operand, validator, target.clone()));
                                    }

                                } else if tcx.is_smart_pointer(impl_ty) {
                                    panic!("No validator for: {}", impl_ty.to_string());
                                }
                            }
                        },
                        _ => {}
                    }
                },
                TerminatorKind::Drop { place,.. } => {
                    let place_ty = place.ty(&body.local_decls, tcx);
                    let actual_ty = place_ty.ty;
                    if let Some(validator) = actual_ty.ty_adt_id().and_then(|id|{
                        tcx.calculate_validator(id)
                    }){
                        drop_validators.insert(idx, (place, validator));
                    }
                },
                _ => {}
            }
        }

        for (idx, data) in validators {
            let bb = bbs.get(idx).unwrap();
            let temp = Place::from(patch.new_temp(Ty::new_tup(tcx, &[]), body.span.clone()));
            let arg_ty = data.0.ty(&body.local_decls, tcx);
            let args = if let ty::Adt(_, args) = arg_ty.peel_refs().kind() {
                *args
            } else {
                List::empty()
            };
            let validator = Operand::function_handle(tcx, data.1.did, args, body.span.clone());
            let block_data = BasicBlockData {
                statements: vec![],
                terminator: Some(Terminator {
                    source_info: SourceInfo::outermost(body.span.clone()),
                    kind: TerminatorKind::Call { func: validator, args: vec![data.0], destination: temp, target: data.2.clone(), unwind: UnwindAction::Continue, call_source: CallSource::Misc, fn_span: body.span.clone() }
                }),
                is_cleanup: false
            };
            
            let new_idx = patch.new_block(block_data);
            let mut orig_terminator = bb.terminator().kind.clone();
            if let TerminatorKind::Call{target,..} = &mut orig_terminator {
                *target = Some(new_idx);
            }
            patch.patch_terminator(idx, orig_terminator);
        }

        patch.apply(body);
    }
}
