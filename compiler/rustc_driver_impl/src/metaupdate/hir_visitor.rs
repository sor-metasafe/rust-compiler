use std::{path::Path, io::BufReader};

use rustc_ast::NodeId;
use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_hir::{intravisit::Visitor, ExprKind, def_id::DefId};
use rustc_middle::{hir::nested_filter::OnlyBodies, ty::{self, TyCtxt}};

use serde::{Serialize, Deserialize};
use serde_json;

pub struct HirAnalysisCtxt<'tcx> {
    tcx: TyCtxt<'tcx>,
    crate_records: FxHashMap<String, CrateRecord>,
    field_def_records: FxHashMap<String, Vec<FieldDefRecord>>
}

#[derive(Serialize, Deserialize)]
struct CrateRecord {
    crate_name: String,
    structs: FxHashMap<usize, StructRecord>
}

#[derive(Serialize, Deserialize)]
struct StructRecord {
    index: usize,
    fields: FxHashMap<usize, FieldRecord>
}

#[derive(Serialize, Deserialize)]
struct FieldRecord {
    struct_index: usize,
    def_index: usize,
    ident: String,
    field_expr_uses: FxHashSet<u32>,
    struct_field_defs: FxHashSet<u32>
}

#[derive(Serialize, Deserialize)]
struct FieldDefRecord {
    def_index: usize,
    node_id: u32,
    ident: String
}

impl<'tcx> HirAnalysisCtxt<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        let crates = tcx.crates(());

        let path = Path::new("/tmp/metasafe/");
        if !path.exists() {
            let _ = std::fs::create_dir(path).unwrap();
        }
        let mut record_file = path.to_path_buf();
        record_file.push("analysis.json");

        let mut crate_records: FxHashMap<String, CrateRecord> = FxHashMap::default();

        if record_file.exists() {
            let file = std::fs::File::open(record_file.as_path()).unwrap();
            let buf_reader = BufReader::new(file);
            crate_records = serde_json::from_reader(buf_reader).unwrap();
        }
        
        let mut this = Self {
            tcx,
            crate_records,
            field_def_records: FxHashMap::default()
        };
        crates.iter().for_each(|crate_num|{
            let crate_name = tcx.crate_name(*crate_num).to_string();
            if !this.crate_records.contains_key(&crate_name) {
                this.add_crate(crate_name);
            }
        });
        this
    }

    fn add_crate(&mut self, name: String){
        let crate_record = CrateRecord {
            crate_name: name.clone(),
            structs: FxHashMap::default()
        };
        self.crate_records.insert(name.clone(), crate_record);
    }

    fn add_special_field_expr_use(&mut self, adt_def: DefId, fdef_index: usize, ident: String, struct_index: usize, fx_usage: NodeId) {
        let crate_name = self.tcx.crate_name(adt_def.krate).to_string();
        let krate = self.crate_records.get_mut(&crate_name).unwrap();
        krate.structs.entry(adt_def.index.as_usize()).and_modify(|struct_entry|{
            struct_entry.fields.entry(struct_index).and_modify(|field_entry|{
                field_entry.field_expr_uses.insert(fx_usage.as_u32());
            }).or_insert_with(||{
                let mut exprs = FxHashSet::default();
                exprs.insert(fx_usage.as_u32());
                FieldRecord { 
                    struct_index, 
                    ident: ident.clone(), 
                    def_index: fdef_index,
                    field_expr_uses: exprs, 
                    struct_field_defs: FxHashSet::default() 
                }
            });
        }).or_insert_with(||{
            let mut struct_record = StructRecord {
                index: adt_def.index.as_usize(),
                fields: FxHashMap::default()
            };

            let mut exprs = FxHashSet::default();
            exprs.insert(fx_usage.as_u32());
            struct_record.fields.insert(struct_index, FieldRecord { struct_index, def_index: fdef_index, ident: ident.clone(), field_expr_uses: exprs, struct_field_defs: FxHashSet::default() });
            struct_record
        });
    }

    /// expressions from struct {i: ..., }
    fn add_struct_field_defs(&mut self, adt_def: DefId, fdef_index: usize, ident: String, struct_index: usize, field_def: NodeId) {
        let crate_name = self.tcx.crate_name(adt_def.krate).to_string();
        let krate = self.crate_records.get_mut(&crate_name).unwrap();
        krate.structs.entry(adt_def.index.as_usize()).and_modify(|struct_entry|{
            struct_entry.fields.entry(struct_index).and_modify(|field_entry|{
                field_entry.field_expr_uses.insert(field_def.as_u32());
            }).or_insert_with(||{
                let mut defs = FxHashSet::default();
                defs.insert(field_def.as_u32());
                FieldRecord { 
                    struct_index, 
                    ident: ident.clone(), 
                    def_index: fdef_index,
                    field_expr_uses: FxHashSet::default(), 
                    struct_field_defs: defs 
                }
            });
        }).or_insert_with(||{
            let mut struct_record = StructRecord {
                index: adt_def.index.as_usize(),
                fields: FxHashMap::default()
            };

            let mut defs = FxHashSet::default();
            defs.insert(field_def.as_u32());
            struct_record.fields.insert(struct_index, FieldRecord { struct_index, def_index: fdef_index, ident, field_expr_uses: FxHashSet::default(), struct_field_defs: defs });
            struct_record
        });
    }

    fn add_field_record(&mut self, crate_name: String, field_ident: String, field_index: usize, field_node_id: NodeId) {
        self.field_def_records.entry(crate_name).and_modify(|entry|{
            entry.push(FieldDefRecord {
                ident: field_ident.clone(),
                def_index: field_index,
                node_id: field_node_id.as_u32()
            });
        }).or_insert_with(||{
            vec![FieldDefRecord {
                ident: field_ident.clone(),
                def_index: field_index,
                node_id: field_node_id.as_u32()
            }]
        });
    }

    fn save_analysis(&self) {
        let path = Path::new("/tmp/metasafe/");
        if !path.exists() {
            let _ = std::fs::create_dir(path).unwrap();
        }

        let mut file = path.to_path_buf();
        file.push("analysis.json");

        let json_string = serde_json::to_string(&self.crate_records).unwrap();
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
    fn visit_expr(&mut self, expr: &'_ rustc_hir::Expr<'_>) {
        match expr.kind {
            ExprKind::Struct(_,fields , _ ) => {
                let tc = self.tcx.typeck(expr.hir_id.owner.def_id);
                if let Some(parent_ty) = tc.node_type_opt(expr.hir_id) {
                    if self.tcx.contains_smart_pointer(parent_ty) {
                        if let ty::Adt(adt, generics) = parent_ty.kind() {
                            let struct_did = adt.did();
                            for (index, field) in adt.all_fields().enumerate() {
                                let field_ty = field.ty(self.tcx, &generics);
                                if !self.tcx.is_smart_pointer(field_ty) {
                                    let field_ident = field.ident(self.tcx).to_string();
                                    let id_map= self.tcx.hir_id_to_node_id.borrow();
                                    let field_expr_id = id_map.get(&fields[index].hir_id).unwrap();
                                    let fdef_index = field.did.index.as_usize();
                                    self.add_struct_field_defs(struct_did, fdef_index,field_ident, index, *field_expr_id);
                                }
                            }
                        }
                    }
                }
            },

            ExprKind::Field(parent, ident) => {
                let tc = self.tcx.typeck(expr.hir_id.owner.def_id);
                if let Some(parent_ty) = tc.node_type_opt(parent.hir_id) {
                    if self.tcx.contains_smart_pointer(parent_ty) {
                        if let Some(field_ty) =  tc.node_type_opt(expr.hir_id) {
                            if !self.tcx.is_smart_pointer(field_ty) {
                                if let ty::Adt(adt, _) = parent_ty.kind() {
                                    for (index, field) in adt.all_fields().enumerate() {
                                        if field.ident(self.tcx) == ident {
                                            let ident = ident.to_string();
                                            let id_map = self.tcx.hir_id_to_node_id.borrow();
                                            let fx_usage = id_map.get(&expr.hir_id).unwrap();
                                            self.add_special_field_expr_use(adt.did(), field.did.index.as_usize(), ident, index, *fx_usage)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },

            _ => {}
        }
        
    }

    fn visit_field_def(&mut self, field_def: &'_ rustc_hir::FieldDef<'_>) {
        let ident = field_def.ident.to_string();
        let field_index = field_def.def_id.to_def_id().index.as_usize();
        let id_map = self.tcx.hir_id_to_node_id.borrow();
        let field_node_id = id_map.get(&field_def.hir_id).unwrap();
        let crate_name = self.tcx.crate_name(field_def.def_id.to_def_id().krate).to_string();
        self.add_field_record(crate_name, ident, field_index, *field_node_id);
    }
}

pub fn run_metasafe_analysis_stage<'tcx>(tcx: TyCtxt<'tcx>) {
    let mut analyzer = HirAnalysisCtxt::new(tcx);
    tcx.hir().visit_all_item_likes_in_crate(&mut analyzer);
    analyzer.save_analysis();
}