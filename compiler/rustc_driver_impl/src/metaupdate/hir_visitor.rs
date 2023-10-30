use rustc_ast::{NodeId, DUMMY_NODE_ID};
use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_hir::{intravisit::Visitor, ItemKind, ExprKind, def_id::DefId, VariantData, Node};
use rustc_middle::{hir::nested_filter::OnlyBodies, ty::{EarlyBinder, self, TypeVisitableExt, TyCtxt}};
use rustc_span::source_map::LocalDefId;

use super::MetaUpdateCallbacks;

pub struct HirAnalysisCtxt<'tcx> {
    tcx: TyCtxt<'tcx>,
    metaupdate_trait_id: Option<DefId>,
    crate_records: FxHashMap<String, CrateRecord>,
    field_def_records: FxHashMap<String, Vec<FieldDefRecord>>
}

struct CrateRecord {
    crate_name: String,
    structs: FxHashMap<usize, StructRecord>
}

struct StructRecord {
    index: usize,
    fields: FxHashMap<usize, FieldRecord>
}

struct FieldRecord {
    struct_index: usize,
    def_index: usize,
    ident: String,
    field_expr_uses: FxHashSet<NodeId>,
    struct_field_uses: FxHashSet<NodeId>
}

struct FieldDefRecord {
    def_index: usize,
    node_id: NodeId,
    ident: String
}

impl<'tcx> HirAnalysisCtxt<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        let crates = tcx.crates(());
        
        let mut this = Self {
            tcx,
            metaupdate_trait_id: tcx.metasafe_metaupdate_trait_id(()),
            crate_records: FxHashMap::default(),
            field_def_records: FxHashMap::default()
        };
        crates.iter().for_each(|crate_num|{
            let crate_name = tcx.crate_name(*crate_num).to_string();
            this.add_crate(crate_name);
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

    fn add_special_field_expr_use(&mut self, adt_def: DefId, fdef_index: NodeId, ident: String, struct_index: usize, fx_usage: NodeId) {
        let crate_name = self.tcx.crate_name(adt_def.krate).to_string();
        let krate = self.crate_records.get_mut(&crate_name).unwrap();
        krate.structs.entry(adt_def.index.as_usize()).and_modify(|struct_entry|{
            struct_entry.fields.entry(index).and_modify(|field_entry|{
                field_entry.field_expr_uses.insert(fx_usage);
            }).or_insert_with(||{
                let mut exprs = FxHashSet::default();
                exprs.insert(fx_usage);
                FieldRecord { 
                    struct_index, 
                    ident, 
                    def_index: fdef_index,
                    field_expr_uses: exprs, 
                    struct_field_uses: FxHashSet::default() 
                }
            })
        }).or_insert_with(||{
            let mut struct_record = StructRecord {
                index: adt_def.index.as_usize(),
                fields: FxHashMap::default()
            };

            let mut exprs = FxHashSet::default();
            exprs.insert(fx_usage);
            struct_record.fields.insert(index, FieldRecord { struct_index, def_index: fdef_index, ident, field_expr_uses: exprs, struct_field_uses: FxHashSet::default() });
            struct_record
        })
    }

    /// expressions from struct {i: ..., }
    fn add_special_struct_field_exprs(&mut self, adt_def: DefId, ident: String, index: usize, fx_usage: NodeId) {

    }

    fn add_field_record(&mut self, crate_name: String, field_ident: String, field_index: usize, field_node_id: NodeId) {
        let def_record = FieldDefRecord {
            ident: field_ident,
            def_index: field_index,
            node_id: field_node_id
        };
        self.field_def_records.entry(crate_name).and_modify(|entry|{
            entry.push(def_record);
        }).or_insert_with(||{
            vec![def_record]
        });
    }

    fn save_analysis(&self) {
        
    }

}

impl Visitor for HirAnalysisCtxt {
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
                    if self.tcx.contains_smart_pointer(parent_ty) {
                        if let ty::Adt(adt, generics) = parent_ty.kind() {
                            let struct_did = adt.did();
                            for (index, field) in adt.all_fields().enumerate() {
                                let field_ty = field.ty(self.tcx, &generics);
                                if !self.tcx.is_smart_pointer(field_ty) {
                                    let field_ident = field.ident(self.tcx).to_string();
                                    let id_map= self.tcx.hir_id_to_node_id.borrow();
                                    let field_expr_id = id_map.get(&fields[index].hir_id).unwrap();
                                    self.add_special_field_expr_use(struct_did, field_ident, index, field_expr_id);
                                }
                            }
                        }
                    }
                }
            },

            ExprKind::Field(parent, ident) => {
                let tc = self.tcx.typeck(expr.hir_id.owner.def_id);
                if let Some(parent_ty) = tc.node_type_opt(parent) {
                    if self.tcx.contains_smart_pointer(parent_ty) {
                        if let Some(field_ty) =  tc.node_type_opt(expr.hir_id) {
                            if !self.tcx.is_smart_pointer(field_ty) {
                                if let ty::Adt(adt, _) = parent_ty.kind() {
                                    for (index, field) in adt.all_fields().enumerate() {
                                        let ident = ident.to_string();

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

    fn visit_field_def(&mut self, field_def: &'v rustc_hir::FieldDef<'v>) {
        let ident = field_def.ident.to_string();
        let field_index = field_def.def_id.to_def_id().index.as_usize();
        let id_map = self.tcx.hir_id_to_node_id.borrow();
        let field_node_id = id_map.get(&field_def.hir_id).unwrap();
        let crate_name = self.tcx.crate_name(field_def.def_id.to_def_id().krate).to_string();
        self.add_field_record(crate_name, ident, field_index, field_node_id);
    }
}

pub fn run_metasafe_analysis_stage<'tcx>(tcx: TyCtxt<'tcx>) {
    let mut analyzer = HirAnalysisCtxt::new(tcx);
    tcx.hir().visit_all_item_likes_in_crate(&mut analyzer);
    analyzer.save_analysis();
}