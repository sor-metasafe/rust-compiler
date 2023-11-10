use std::ops::DerefMut;

use ast::{PathSegment, AngleBracketedArgs, AngleBracketedArg, AnonConst, Expr, ExprKind, StructExpr, ExprField};
use rustc_ast::{self as ast, mut_visit::MutVisitor, GenericArgs, GenericArg, ptr::P, Item, ItemKind, NodeId, VariantData, FieldDef, Visibility, VisibilityKind, Ty, TyKind, DUMMY_NODE_ID};
use rustc_data_structures::fx::FxHashSet;
use rustc_span::{symbol::Ident, Symbol, DUMMY_SP, LocalExpnId};
use smallvec::{SmallVec, smallvec};
use thin_vec::thin_vec;
use rustc_expand::base::ResolverExpand;

use crate::load_analysis;

pub struct AstMutVisitor<'a> {
    boxable_structs: FxHashSet<NodeId>,
    special_struct_defs: FxHashSet<NodeId>,
    except_struct_defs: FxHashSet<NodeId>,
    resolver: &'a mut dyn ResolverExpand,
    expn_id: LocalExpnId
}


impl<'a> AstMutVisitor<'a> {
    pub fn new(crate_name: String, resolver: &'a mut dyn ResolverExpand) -> Self {
        let analysis_records = load_analysis(crate_name.clone());
        Self {
            boxable_structs: analysis_records.structs.clone(),
            special_struct_defs: analysis_records.struct_defs.clone(),
            except_struct_defs: analysis_records.except_defs.clone(),
            resolver,
            expn_id: LocalExpnId::fresh_empty()
        }
    }
}

fn visit_expr_inner<'a>(Expr {id, kind, span: _, attrs: _, tokens: _}: &mut Expr, this: &mut AstMutVisitor<'a>) {
    match kind {
        ExprKind::Struct(s) => {
            if this.special_struct_defs.contains(&id) && !this.except_struct_defs.contains(&id) {
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
                let expn_id = LocalExpnId::fresh_empty();
                let fragment = AstFragment::ExprFields(fields); // ::Expr()
                this.resolver.visit_ast_fragment_with_placeholders(expn_id,&fragment);
            }
        },
        _ => {}
    }
}

impl<'a> MutVisitor for AstMutVisitor<'a> {
    fn flat_map_item(&mut self, mut item: P<Item>) -> SmallVec<[P<Item>; 1]> {
        let Item { ident: _, attrs: _, id, kind, vis: _, span: _, tokens: _ } = item.deref_mut();
        match kind {
            ItemKind::Struct(vdata, _) => {
                if self.boxable_structs.contains(id) {
                    if let VariantData::Struct(fields , _) = vdata {
                        let field_id = self.resolver.next_node_id();
                        self.resolver.create_d
                        let new_field = FieldDef {
                            ident: Some(Ident::from_str("metasafe_box")),
                            attrs: thin_vec![],
                            id: field_id,
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
        let mut item = smallvec![item];
        let expn_id = LocalExpnId::fresh_empty();
        let fragment = AstFragment::Items(item);
        item = self.resolver.visit_ast_fragment_with_placeholders(expn_id, &fragment);
    }

    fn visit_expr(&mut self, expr: &mut P<Expr>) {
        visit_expr_inner(expr, self);
    }
}
