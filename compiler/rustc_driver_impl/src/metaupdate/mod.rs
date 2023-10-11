use rustc_ast::NodeId;
use rustc_data_structures::fx::FxHashMap;
use rustc_hir::HirId;

mod callbacks;
mod hir_visitor;
mod ast_mut_visitor;

/// This is the driver for MetaSafe compilation.
/// Of course it is not alone, we do some internal modifications
/// For example, we need to hook the HIR type analysis collector to 
/// handle smart pointer housing structs... Hmmm :(
pub struct MetaUpdateCallbacks {
    analysis_done: bool,
    astid2hirid_map: FxHashMap<NodeId, HirId>
}