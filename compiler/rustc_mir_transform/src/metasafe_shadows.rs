//! This pass inserts the call to initialize MetaSafe shadow memory

use rustc_middle::{mir::{MirPass, patch::MirPatch, Operand, BasicBlockData, Terminator, SourceInfo, TerminatorKind, Place, START_BLOCK}, ty::{Ty, TyCtxt}};


pub struct AddMetaSafeShadows;

impl<'tcx> MirPass<'tcx> for AddMetaSafeShadows {
    fn is_enabled(&self, _sess: &rustc_session::Session) -> bool {
        _sess.opts.unstable_opts.metaupdate
    }

    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut rustc_middle::mir::Body<'tcx>) {
        let def_id = body.source.def_id();
        if let Some((main_did, _)) = tcx.entry_fn(()) {
            if def_id == main_did && def_id.is_local() {
                //let's find the first statement and call init the shadow memory
                let mut patch = MirPatch::new(body);
                let temp = Place::from(patch.new_temp(Ty::new_unit(tcx), body.span));
                let shadow_function = Operand::function_handle(tcx, tcx.require_lang_item(rustc_hir::LangItem::MetaSafeShadowAlloc, None), [], body.span);
                //let bbs = &body.basic_blocks;
                let block_data = BasicBlockData {
                    statements: vec![],
                    terminator: Some(Terminator {
                        source_info: SourceInfo::outermost(body.span),
                        kind: TerminatorKind::Call { 
                            func: shadow_function, 
                            args: vec![], 
                            destination: temp, 
                            target: None, 
                            unwind: rustc_middle::mir::UnwindAction::Continue, 
                            call_source: rustc_middle::mir::CallSource::Misc, 
                            fn_span: body.span }
                    }),
                    is_cleanup: false
                };
                let block = patch.new_block(block_data);
                patch.apply(body);

                let bbs = body.basic_blocks_mut();
                if bbs.len() > 1 {
                    bbs.swap(START_BLOCK, block);
                    let entry = bbs.get_mut(START_BLOCK).unwrap();
                    let terminator = entry.terminator_mut();
                    if let TerminatorKind::Call { target, ..} = &mut terminator.kind {
                        target.replace(block);
                    }
                }
            }
        }
    }
}