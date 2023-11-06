use std::ops::DerefMut;

use ast::{PathSegment, AngleBracketedArgs, AngleBracketedArg, AnonConst, Expr, ExprKind, StructExpr, ExprField};
use rustc_ast::{self as ast, mut_visit::MutVisitor, GenericArgs, GenericArg, ptr::P, Item, ItemKind, NodeId, VariantData, FieldDef, Visibility, VisibilityKind, Ty, TyKind};
use rustc_data_structures::fx::FxHashSet;
use rustc_span::{symbol::Ident, Symbol, DUMMY_SP};
use smallvec::{SmallVec, smallvec};
use thin_vec::thin_vec;
use rustc_resolve::Resolver;
use rustc_expand::base::ResolverExpand;

use crate::load_analysis;

pub struct AstMutVisitor<'a, 'tcx> {
    boxable_structs: FxHashSet<NodeId>,
    special_struct_defs: FxHashSet<NodeId>,
    except_struct_defs: FxHashSet<NodeId>,
    resolver: &'a mut Resolver<'a, 'tcx>
}


impl<'a,'tcx> AstMutVisitor<'a,'tcx> {
    pub fn new(crate_name: String, resolver: &'a mut Resolver<'a,'tcx>) -> Self {
        let analysis_records = load_analysis(crate_name.clone());
        Self {
            boxable_structs: analysis_records.structs.clone(),
            special_struct_defs: analysis_records.struct_defs.clone(),
            except_struct_defs: analysis_records.except_defs.clone(),
            resolver
        }
    }
}

fn visit_expr_inner<'a,'tcx>(Expr {id, kind, span: _, attrs: _, tokens: _}: &mut Expr, this: &mut AstMutVisitor<'a, 'tcx>) {
    match kind {
        ExprKind::Struct(s) => {
            if this.special_struct_defs.contains(&id) && !this.except_struct_defs.contains(&id) {
                let StructExpr {qself: _,fields, path: _, rest: _} = s.deref_mut();

                let array_expr = Expr {
                    id: this.resolver.next_node_id(),
                    kind: ExprKind::Repeat(
                        P(Expr {
                            id: this.resolver.next_node_id(),
                            kind: ExprKind::Lit(ast::token::Lit { kind: ast::token::LitKind::Integer, symbol: Symbol::intern("0"), suffix: None }),
                            span: DUMMY_SP,
                            attrs: thin_vec![],
                            tokens: None
                        }), AnonConst { id: this.resolver.next_node_id(), value: P(
                            Expr {
                                id: this.resolver.next_node_id(),
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
                    id: this.resolver.next_node_id(),
                    span: DUMMY_SP,
                    expr: P(
                        Expr {
                            id: this.resolver.next_node_id(),
                            kind: ExprKind::Call(
                                P(Expr {
                                        id: this.resolver.next_node_id(),
                                        kind: ExprKind::Path(None, ast::Path {
                                            span: DUMMY_SP,
                                            segments: thin_vec![
                                                PathSegment {
                                                    id: this.resolver.next_node_id(),
                                                    ident: Ident::from_str("Box"),
                                                    args: None
                                                },
                                                PathSegment {
                                                    id: this.resolver.next_node_id(),
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

impl<'a,'tcx> MutVisitor for AstMutVisitor<'a,'tcx> {
    fn flat_map_item(&mut self, mut item: P<Item>) -> SmallVec<[P<Item>; 1]> {
        let Item { ident: _, attrs: _, id, kind, vis: _, span: _, tokens: _ } = item.deref_mut();
        match kind {
            ItemKind::Struct(vdata, _) => {
                if self.boxable_structs.contains(id) {
                    if let VariantData::Struct(fields , _) = vdata {
                        let new_field = FieldDef {
                            ident: Some(Ident::from_str("metasafe_box")),
                            attrs: thin_vec![],
                            id: self.resolver.next_node_id(),
                            span: DUMMY_SP,
                            vis: Visibility {
                                kind: VisibilityKind::Public,
                                span: DUMMY_SP,
                                tokens: None
                            },
                            ty: P(Ty{
                                id: self.resolver.next_node_id(),
                                kind: TyKind::Path(None, ast::Path {
                                                            segments: thin_vec![
                                                                PathSegment {
                                                                    ident: Ident::from_str("Box"),
                                                                    id: self.resolver.next_node_id(),
                                                                    args: Some(P(GenericArgs::AngleBracketed(AngleBracketedArgs{
                                                                        span: DUMMY_SP,
                                                                        args: thin_vec![AngleBracketedArg::Arg(GenericArg::Type(
                                                                            P(Ty {
                                                                                id: self.resolver.next_node_id(),
                                                                                kind: TyKind::Array(P(Ty {
                                                                                                        id: self.resolver.next_node_id(), 
                                                                                                        kind: TyKind::Path(None, ast::Path {
                                                                                                            segments: thin_vec![
                                                                                                                PathSegment {
                                                                                                                    ident: Ident::from_str("u8"),
                                                                                                                    id: self.resolver.next_node_id(),
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
                                                                                                        id: self.resolver.next_node_id(),
                                                                                                        value: P(Expr {
                                                                                                            id: self.resolver.next_node_id(),
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