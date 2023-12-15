use std::{io::BufReader, path::Path};

use hir::ItemKind;
use rustc_ast::NodeId;
use rustc_hir::{
    self as hir,
    def_id::{DefId, LOCAL_CRATE},
    intravisit::Visitor,
    ExprKind,
};
use rustc_middle::{
    hir::nested_filter::OnlyBodies,
    ty::{self, TyCtxt},
};
use rustc_data_structures::fx::{FxHashMap,FxHashSet};
use rustc_target::spec::abi::Abi;

use serde::{Deserialize, Serialize};
use serde_json;

pub struct HirAnalysisCtxt<'tcx> {
    tcx: TyCtxt<'tcx>,
    curr_crate_name: String,
    struct_records: FxHashMap<String, FxHashMap<usize, StructRecord>>,
    extern_calls: FxHashMap<String, FxHashSet<u32>>,
    crate_name: String, //  the current crate name
}

#[derive(Serialize, Deserialize, Default)]
pub struct StructRecord {
    pub def_index: usize,
    pub node_id: u32,
    pub needs_box: bool,
    pub struct_defs: FxHashMap<String, FxHashSet<u32>>,
    pub except_defs: FxHashMap<String, FxHashSet<u32>>,
}

impl<'tcx> HirAnalysisCtxt<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        let crates = tcx.crates(());

        let path = Path::new("/tmp/metasafe/");
        if !path.exists() {
            let _ = std::fs::create_dir(path).unwrap();
        }
        let mut record_file = path.to_path_buf();
        record_file.push("struct_records.json");
        let mut extern_calls_file = path.to_path_buf();
        extern_calls_file.push("exten_calls.json");

        let mut struct_records = FxHashMap::default();
        if record_file.exists() {
            let file = std::fs::File::open(record_file.as_path()).unwrap();
            let buf_reader = BufReader::new(file);
            struct_records = serde_json::from_reader(buf_reader).unwrap();
        }

        let mut extern_calls = FxHashMap::default();
        if extern_calls_file.exists() {
            let file = std::fs::File::open(extern_calls_file.as_path()).unwrap();
            let buf_reader = BufReader::new(file);
            extern_calls = serde_json::from_reader(buf_reader).unwrap();
        }

        let mut this = Self {
            tcx,
            curr_crate_name: tcx.crate_name(LOCAL_CRATE).to_string(),
            struct_records,
            crate_name: tcx.crate_name(LOCAL_CRATE).to_string(),
            extern_calls
        };

        crates.iter().for_each(|crate_num| {
            let crate_name = tcx.crate_name(*crate_num).to_string();
            if !this.struct_records.contains_key(&crate_name) {
                this.add_crate(crate_name);
            }
        });
        this
    }

    fn add_crate(&mut self, name: String) {
        self.struct_records.insert(name.clone(), FxHashMap::default());
    }

    fn add_struct(&mut self, node_id: NodeId, def_id: usize) {
        let crate_name = self.curr_crate_name.clone();
        let struct_records = self.struct_records.entry(crate_name).or_default();
        let struct_record = struct_records.entry(def_id).or_default();

        struct_record.def_index = def_id;
        struct_record.node_id = node_id.as_u32();
    }

    fn mark_struct_boxable(&mut self, struct_did: DefId) {
        let crate_name = self.tcx.crate_name(struct_did.krate).to_string();
        self.struct_records.entry(crate_name).and_modify(|map| {
            map.entry(struct_did.index.as_usize()).and_modify(|rec| rec.needs_box = true);
        });
    }

    fn add_struct_def(&mut self, struct_id: DefId, def_id: NodeId) {
        let crate_name = self.tcx.crate_name(struct_id.krate).to_string();
        self.struct_records.entry(crate_name).and_modify(|map| {
            map.entry(struct_id.index.as_usize()).and_modify(|record| {
                let defs = record.struct_defs.entry(self.crate_name.clone()).or_default();
                defs.insert(def_id.as_u32());
            });
        });
    }

    fn set_struct_def_exception(&mut self, struct_did: DefId, expr_id: NodeId) {
        let crate_name = self.tcx.crate_name(struct_did.krate).to_string();
        let struct_records = self.struct_records.entry(crate_name).or_default();
        let record = struct_records.entry(struct_did.index.as_usize()).or_default();
        record.needs_box = true;
        let except_defs = record.except_defs.entry(self.curr_crate_name.clone()).or_default();
        except_defs.insert(expr_id.as_u32());
    }

    fn add_extern_call(&mut self, expr_id: NodeId) {
        let crate_name = self.tcx.crate_name(LOCAL_CRATE).to_string();
        self.extern_calls.entry(crate_name).and_modify(|set|{
            set.insert(expr_id.as_u32());
        }).or_insert_with(||{
            let mut set = FxHashSet::default();
            set.insert(expr_id.as_u32());
            set
        });
    }

    fn save_analysis(&self) {
        let path = Path::new("/tmp/metasafe/");
        if !path.exists() {
            let _ = std::fs::create_dir(path).unwrap();
        }

        let mut file = path.to_path_buf();
        file.push("struct_records.json");

        let json_string = serde_json::to_string(&self.struct_records).unwrap();
        std::fs::write(file, json_string).unwrap();

        file = path.to_path_buf();
        file.push("extern_calls.json");
        let json_string = serde_json::to_string(&self.extern_calls).unwrap();
        std::fs::write(file, json_string).unwrap();
    }
}

impl<'tcx> Visitor<'tcx> for HirAnalysisCtxt<'tcx> {
    type NestedFilter = OnlyBodies;

    /// Nested map visiting
    fn nested_visit_map(&mut self) -> Self::Map {
        self.tcx.hir()
    }

    /// Visit an HIR expression. This should help identify which generic types end up
    /// taking smart pointer generics
    fn visit_expr(&mut self, expr: &'tcx rustc_hir::Expr<'tcx>) {
        match expr.kind {
            ExprKind::Struct(_, _, _) => {
                let tc = self.tcx.typeck(expr.hir_id.owner.def_id);
                if let Some(parent_ty) = tc.node_type_opt(expr.hir_id) {
                    let parent_ty = parent_ty.peel_refs();
                    if let ty::Adt(adt, _) = parent_ty.kind() {
                        let struct_did = adt.did();
                        if self.tcx.contains_smart_pointer(parent_ty) {
                            self.mark_struct_boxable(struct_did);
                        }
                        let id_map = self.tcx.hir_id_to_node_id.borrow();
                        let expr_id = id_map.get(&expr.hir_id).unwrap();
                        self.add_struct_def(struct_did, *expr_id);
                    }
                }
            }

            ExprKind::Assign(lhs, _, _) => {
                if let ExprKind::Struct(_, _, _) = lhs.kind {
                    let tc = self.tcx.typeck(expr.hir_id.owner.def_id);
                    if let Some(lhs_ty) = tc.node_type_opt(lhs.hir_id) {
                        if self.tcx.contains_smart_pointer(lhs_ty) {
                            let lhs_ty = lhs_ty.peel_refs();
                            if let ty::Adt(adt, _) = lhs_ty.kind() {
                                let struct_id = adt.did();
                                let id_map = self.tcx.hir_id_to_node_id.borrow();
                                let expr_id = id_map.get(&lhs.hir_id).unwrap();
                                self.set_struct_def_exception(struct_id, *expr_id);
                            }
                        }
                    }
                }
            }

            ExprKind::Field(parent, _) => {
                let tc = self.tcx.typeck(expr.hir_id.owner.def_id);
                if let Some(parent_ty) = tc.node_type_opt(parent.hir_id) {
                    let parent_ty = parent_ty.peel_refs();
                    if self.tcx.contains_smart_pointer(parent_ty) {
                        if let ty::Adt(adt_def, _) = parent_ty.kind() {
                            let struct_did = adt_def.did();
                            self.mark_struct_boxable(struct_did);
                        }
                    }
                }
            },
            ExprKind::Call(callee, _args) => {
                let tc = self.tcx.typeck(expr.hir_id.owner.def_id);
                if let Some(callee_ty) = tc.node_type_opt(callee.hir_id) {
                    match callee_ty.kind() {
                        ty::FnDef(def_id, args) => {
                            let fn_sig = self.tcx.fn_sig(def_id).instantiate(self.tcx, args);
                            match fn_sig.abi() {
                                Abi::Cdecl { unwind: _ } | Abi::C { unwind: _ } => {
                                    //Now we have these functions which we believe are FFI.
                                    //Question is whether even functions defined in Rust shared to
                                    //C require execution on different stack. Aren't they aware of
                                    //smart pointers, for example?
                                    //TODO: make sure to only include functions with no bodies.
                                    let node_id_map = self.tcx.hir_id_to_node_id.borrow();
                                    let node_id = node_id_map.get(&expr.hir_id).unwrap();
                                    self.add_extern_call(*node_id);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        hir::intravisit::walk_expr(self, expr);
    }

    // fn visit_field_def(&mut self, field_def: &'tcx rustc_hir::FieldDef<'tcx>) {
    //     let ident = field_def.ident.to_string();
    //     let field_index = field_def.def_id.to_def_id().index.as_usize();
    //     let id_map = self.tcx.hir_id_to_node_id.borrow();
    //     let field_node_id = id_map.get(&field_def.hir_id).unwrap();
    //     let crate_name = self.tcx.crate_name(field_def.def_id.to_def_id().krate).to_string();
    //     self.add_field_record(crate_name, ident, field_index, *field_node_id);
    //     hir::intravisit::walk_field_def(self, field_def);
    // }

    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) {
        match item.kind {
            ItemKind::Struct(_, _) => {
                let id_map = self.tcx.hir_id_to_node_id.borrow();
                let struct_node_id = id_map.get(&item.hir_id()).unwrap();
                let struct_did = item.owner_id.def_id.local_def_index.as_usize();
                self.add_struct(*struct_node_id, struct_did);
            }
            _ => {}
        }
        hir::intravisit::walk_item(self, item);
    }
}

pub fn run_metasafe_analysis_stage<'tcx>(tcx: TyCtxt<'tcx>) {
    let crate_name = tcx.crate_name(LOCAL_CRATE).to_string();
    println!("Crate: {}, ID map size: {}", &crate_name, tcx.hir_id_to_node_id.borrow().len());
    let mut analyzer = HirAnalysisCtxt::new(tcx);
    tcx.hir().visit_all_item_likes_in_crate(&mut analyzer);
    analyzer.save_analysis();
}
