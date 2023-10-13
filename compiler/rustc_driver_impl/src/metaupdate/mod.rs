use std::{path::{Path, PathBuf}, fs::create_dir_all};

use rustc_ast::NodeId;
use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_hir::{HirId, OwnerId};
use rustc_middle::ty::TyCtxt;

mod callbacks;
mod hir_visitor;
mod ast_mut_visitor;

/// This is the driver for MetaSafe compilation.
/// Of course it is not alone, we do some internal modifications
/// For example, we need to hook the HIR type analysis collector to 
/// handle smart pointer housing structs... Hmmm :(
pub struct MetaUpdateCallbacks<'tcx> {
    analysis_done: bool,
    node_id_to_hir_id: FxHashMap<NodeId, HirId>,
    next_node_id: NodeId,
    tcx: TyCtxt<'tcx>
}

impl<'tcx> MetaUpdateCallbacks {
    fn new(tcx: TyCtxt<'tcx>, analysis_done: bool) -> Self {
        Self { analysis_done, node_id_to_hir_id: FxHashMap::default(), next_node_id: FxHashMap::default(), tcx }
    }

    fn add_special_field(&mut self) {

    }

    fn add_generic_field(&mut self) {
        
    }

    fn load_previous_analyses() {
        let mut metaupdate_path = Path::new("/tmp/metaupdate");
        if !metaupdate_path.exists() {
            create_dir_all(metaupdate_path.clone()).expect("Failed to create analysis paths");
            return;
        }

        let mut path = metaupdate_path.to_path_buf();
        path.push("special_types.json");

    }
}