use std::{path::Path, io::BufReader, ops::DerefMut};

use ast::{PathSegment, AngleBracketedArgs, AngleBracketedArg, AnonConst, Expr, ExprKind, StructExpr, ExprField};
use rustc_ast::{self as ast, mut_visit::MutVisitor, GenericArgs, GenericArg, ptr::P, Item, ItemKind, NodeId, VariantData, FieldDef, DUMMY_NODE_ID, Visibility, VisibilityKind, Ty, TyKind};
use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_span::{symbol::Ident, Symbol, DUMMY_SP};
use smallvec::{SmallVec, smallvec};
use thin_vec::{thin_vec};

use super::hir_visitor::StructRecord;

pub struct AstMutVisitor {
    boxable_structs: FxHashSet<NodeId>,
    special_struct_defs: FxHashSet<NodeId>
}


impl AstMutVisitor {
    pub fn new(crate_name: String) -> Self {
        let path = Path::new("/tmp/metasafe/analysis.json");
        let mut boxable_strucs: FxHashSet<NodeId> = FxHashSet::default();
        let mut special_defs: FxHashSet<NodeId> = FxHashSet::default();
        if path.exists() {
            let record_file = std::fs::File::open(path).unwrap();
            let buf_reader = BufReader::new(record_file);
            let records_map: FxHashMap<String, FxHashMap<usize, StructRecord>> = serde_json::from_reader(buf_reader).unwrap(); 
            let this_crate_record = records_map.get(&crate_name).unwrap();
            for (_, record) in this_crate_record {
                if record.needs_box {
                    boxable_strucs.insert(NodeId::from_u32(record.node_id));
                    for id in &record.struct_defs {
                        special_defs.insert(NodeId::from_u32(*id));
                    }
                }
            }
        }

        Self {
            boxable_structs: boxable_strucs,
            special_struct_defs: special_defs
        }
    }
}

fn visit_expr_inner(Expr {id, kind, span: _, attrs: _, tokens: _}: &mut Expr, this: &mut AstMutVisitor) {
    match kind {
        ExprKind::Struct(s) => {
            if this.special_struct_defs.contains(&id) {
                let StructExpr {qself: _,fields, path: _, rest: _} = s.deref_mut();
                
                let array_expr = Expr {
                    id: DUMMY_NODE_ID,
                    kind: ExprKind::Repeat(
                        P(Expr {
                            id: DUMMY_NODE_ID,
                            kind: ExprKind::Lit(ast::token::Lit { kind: ast::token::LitKind::Integer, symbol: Symbol::intern("0"), suffix: None }),
                            span: DUMMY_SP,
                            attrs: thin_vec![],
                            tokens: None
                        }), AnonConst { id: DUMMY_NODE_ID, value: P(
                            Expr {
                                id: DUMMY_NODE_ID,
                                kind: ExprKind::Lit(ast::token::Lit { kind: ast::token::LitKind::Integer, symbol: Symbol::intern("0"), suffix: None }),
                                span: DUMMY_SP,
                                attrs: thin_vec![],
                                tokens: None
                            }
                        ) }),
                    span: DUMMY_SP,
                    attrs: thin_vec![],
                    tokens: None
                };
                
                let expr_field = ExprField {
                    ident: Ident::from_str("metasafe_box"),
                    attrs: thin_vec![],
                    id: DUMMY_NODE_ID,
                    span: DUMMY_SP,
                    expr: P(
                        Expr {
                            id: DUMMY_NODE_ID,
                            kind: ExprKind::Call(
                                P(Expr {
                                        id: DUMMY_NODE_ID,
                                        kind: ExprKind::Path(None, ast::Path {
                                            span: DUMMY_SP,
                                            segments: thin_vec![
                                                PathSegment {
                                                    id: DUMMY_NODE_ID,
                                                    ident: Ident::from_str("Box"),
                                                    args: None
                                                },
                                                PathSegment {
                                                    id: DUMMY_NODE_ID,
                                                    ident: Ident::from_str("new"),
                                                    args: None
                                                }
                                            ],
                                            tokens: None
                                        }),
                                        span: DUMMY_SP,
                                        attrs: thin_vec![],
                                        tokens: None
                                    }
                                ),
                                thin_vec![P(array_expr)]),
                            span: DUMMY_SP,
                            attrs: thin_vec![],
                            tokens: None
                        }
                    ),
                    is_shorthand: false,
                    is_placeholder: false,
                };

                fields.push(expr_field);
            }
        },
        _ => {}
    }
}

impl MutVisitor for AstMutVisitor {
    fn flat_map_item(&mut self, mut item: P<Item>) -> SmallVec<[P<Item>; 1]> {
        let Item { ident: _, attrs: _, id, kind, vis: _, span: _, tokens: _ } = item.deref_mut();
        match kind {
            ItemKind::Struct(vdata, _) => {
                if self.boxable_structs.contains(id) {
                    if let VariantData::Struct(fields , _) = vdata {
                        let new_field = FieldDef {
                            ident: Some(Ident::from_str("metasafe_box")),
                            attrs: thin_vec![],
                            id: DUMMY_NODE_ID,
                            span: DUMMY_SP,
                            vis: Visibility {
                                kind: VisibilityKind::Public,
                                span: DUMMY_SP,
                                tokens: None
                            },
                            ty: P(Ty{
                                id: DUMMY_NODE_ID,
                                kind: TyKind::Path(None, ast::Path {
                                                            segments: thin_vec![
                                                                PathSegment {
                                                                    ident: Ident::from_str("Box"),
                                                                    id: DUMMY_NODE_ID,
                                                                    args: Some(P(GenericArgs::AngleBracketed(AngleBracketedArgs{
                                                                        span: DUMMY_SP,
                                                                        args: thin_vec![AngleBracketedArg::Arg(GenericArg::Type(
                                                                            P(Ty {
                                                                                id: DUMMY_NODE_ID,
                                                                                kind: TyKind::Array(P(Ty {
                                                                                                        id: DUMMY_NODE_ID, 
                                                                                                        kind: TyKind::Path(None, ast::Path {
                                                                                                            segments: thin_vec![
                                                                                                                PathSegment {
                                                                                                                    ident: Ident::from_str("u8"),
                                                                                                                    id: DUMMY_NODE_ID,
                                                                                                                    args: None,
                                                                                                                }
                                                                                                            ],
                                                                                                            span: DUMMY_SP,
                                                                                                            tokens: None
                                                                                                        }),
                                                                                                        span: DUMMY_SP,
                                                                                                        tokens: None
                                                                                                    }), 
                                                                                                    AnonConst {
                                                                                                        id: DUMMY_NODE_ID,
                                                                                                        value: P(Expr {
                                                                                                            id: DUMMY_NODE_ID,
                                                                                                            kind: ExprKind::Lit(ast::token::Lit { 
                                                                                                                kind: ast::token::LitKind::Integer,
                                                                                                                symbol: Symbol::intern("32"),
                                                                                                                suffix: None
                                                                                                            }),
                                                                                                            span: DUMMY_SP,
                                                                                                            attrs: thin_vec![],
                                                                                                            tokens: None
                                                                                                        })
                                                                                                    }),
                                                                                span: DUMMY_SP,
                                                                                tokens: None
                                                                            })
                                                                        ))]
                                                                    })))
                                                                }
                                                            ],
                                                            span: DUMMY_SP,
                                                            tokens: None
                                                        }),
                                span: DUMMY_SP,
                                tokens: None
                            }),
                            is_placeholder: false,
                        };
                        fields.push(new_field);
                    }
                }
            },
            _ => {}
        }
        smallvec![item]
    }

    fn visit_expr(&mut self, expr: &mut P<Expr>) {
        visit_expr_inner(expr, self);
    }
}