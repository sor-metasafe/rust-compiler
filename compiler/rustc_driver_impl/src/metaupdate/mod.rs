use std::{path::{Path, PathBuf}, fs::create_dir_all, collections::hash_map::Entry};

use rustc_ast::NodeId;
use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_hir::{HirId, OwnerId};
use rustc_middle::ty::{TyCtxt, Ty};
use rustc_span::sym::index;

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
    movable_fields: FxHashMap<String, FxHashMap<DefId, FxHashMap<usize, (bool, FxHashSet<Ty<'tcx>>)>>>, // crate_name: {struct_id: {fields}}
    tcx: TyCtxt<'tcx>
}

struct MovableField {
    hir_id: HirId,
    index: usize,
    generics: Option<HirId>
}

impl<'tcx> MetaUpdateCallbacks<'tcx> {
    fn new(tcx: TyCtxt<'tcx>, analysis_done: bool) -> Self {
        Self { analysis_done, node_id_to_hir_id: FxHashMap::default(), next_node_id: FxHashMap::default(), movable_fields: FxHashMap::default(), tcx }
    }

    fn add_special_field(&mut self, struct_id: DefId, field_index: usize, field_ty: Ty<'tcx>) {
        let krate_name = self.tcx.crate_name(struct_id.krate).to_string();
        match self.movable_fields.entry(krate_name) {
            Entry::Occupied(o) => {
                let map = o.get_mut();
                match map.entry(struct_id) {
                    Entry::Occupied(o) => {
                        let map = o.get_mut();
                        match map.entry(field_index) {
                            Entry::Occupied(o) => {
                                let (_, set) = o.get_mut(); //.insert(field_ty);
                                set.insert(field_ty)
                            },
                            Entry::Vacant(v) => {
                                let mut map = FxHashMap::default();
                                let mut set = FxHashSet::default();
                                set.insert((false, field_ty));
                                map.insert(field_index, set);
                            }
                        }
                    },
                    Entry::Vacant(v) => {
                        let mut map = FxHashMap::default();
                        let mut set = FxHashSet::default();
                        set.insert((false, field_ty));
                        map.insert(field_index, set);
                        v.insert(map);
                    }
                }
            },
            Entry::Vacant(v) => {
                let mut structs_map = FxHashMap::default();
                let mut map = FxHashMap::default();
                let mut set = FxHashSet::default();
                set.insert(field_ty);
                map.insert(index, (false, set));
                structs_map.insert(struct_id, map);
                v.insert(structs_map)
            }
        }
    }

    fn add_generic_field(&mut self) {
        
    }

    fn add_smart_field(&mut self, struct_id: DefId, field_index: usize) {
        let krate_name = self.tcx.crate_name(struct_id.krate).to_string();
        match self.movable_fields.entry(krate_name) {
            Entry::Occupied(o) => {
                match o.get_mut().entry(struct_id) {
                    Entry::Occupied(o) => {
                        match o.get_mut().entry(field_index) {
                            Entry::Occupied(o) => {
                                let (smart, _) = o.get_mut();
                                *smart = true;
                            },
                            Entry::Vacant(v) => {
                                v.insert((true, FxHashSet::default()))
                            }
                        }
                    },
                    Entry::Vacant(v) => {
                        let mut map = FxHashMap::default();
                        map.insert(field_index, (true, FxHashSet::default()));
                        v.insert(map);
                    }
                }
            },
            Entry::Vacant(v) => {
                let mut structs_map = FxHashMap::default();
                let mut fields_map = FxHashMap::default();
                let mut set = FxHashSet::default();
                fields_map.insert(field_index, (true, set));
                structs_map.insert(struct_id, fields_map);
                v.insert(structs_map);
            }
        }
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

    fn save_analysis(&self){
        
    }
}