use ast::{Closure, Expr, ExprKind, FnDecl};
use rustc_ast::{self as ast, mut_visit::MutVisitor, ptr::P, NodeId};
use rustc_data_structures::fx::FxHashSet;
use rustc_expand::{
    base::{ExtCtxt, ResolverExpand},
    expand::{AstFragment, ExpansionConfig},
};
use rustc_feature::Features;
use rustc_session::Session;
use rustc_span::{
    symbol::{sym, Ident},
    DUMMY_SP,
};
//use smallvec::{smallvec, SmallVec};
//use std::ops::DerefMut;
use thin_vec::{thin_vec, ThinVec};

use crate::load_extern_calls;
/*
pub struct AstMutVisitor<'a> {
    boxable_structs: FxHashSet<NodeId>,
    special_struct_defs: FxHashSet<NodeId>,
    except_struct_defs: FxHashSet<NodeId>,
    resolver: &'a mut dyn ResolverExpand,
    expn_id: LocalExpnId,
}

impl<'a> AstMutVisitor<'a> {
    pub fn new(crate_name: String, resolver: &'a mut dyn ResolverExpand) -> Self {
        let analysis_records = load_struct_records_analysis(crate_name.clone());
        Self {
            boxable_structs: analysis_records.structs.clone(),
            special_struct_defs: analysis_records.struct_defs.clone(),
            except_struct_defs: analysis_records.except_defs.clone(),
            resolver,
            expn_id: LocalExpnId::fresh_empty(),
        }
    }
}

fn visit_expr_inner<'a>(
    Expr { id, kind, span: _, attrs: _, tokens: _ }: &mut Expr,
    this: &mut AstMutVisitor<'a>,
) {
    match kind {
        ExprKind::Struct(s) => {
            let StructExpr { qself: _, fields, path: _, rest: _ } = s.deref_mut();

            let array_expr = Expr {
                id: DUMMY_NODE_ID,
                kind: ExprKind::Repeat(
                    P(Expr {
                        id: DUMMY_NODE_ID,
                        kind: ExprKind::Lit(ast::token::Lit {
                            kind: ast::token::LitKind::Integer,
                            symbol: Symbol::intern("0"),
                            suffix: None,
                        }),
                        span: DUMMY_SP,
                        attrs: thin_vec![],
                        tokens: None,
                    }),
                    AnonConst {
                        id: DUMMY_NODE_ID,
                        value: P(Expr {
                            id: DUMMY_NODE_ID,
                            kind: ExprKind::Lit(ast::token::Lit {
                                kind: ast::token::LitKind::Integer,
                                symbol: Symbol::intern("0"),
                                suffix: None,
                            }),
                            span: DUMMY_SP,
                            attrs: thin_vec![],
                            tokens: None,
                        }),
                    },
                ),
                span: DUMMY_SP,
                attrs: thin_vec![],
                tokens: None,
            };

            let expr_field = ExprField {
                ident: Ident::from_str("metasafe_shadow_ptr"),
                attrs: thin_vec![],
                id: DUMMY_NODE_ID,
                span: DUMMY_SP,
                expr: P(Expr {
                    id: DUMMY_NODE_ID,
                    kind: ExprKind::Call(
                        P(Expr {
                            id: DUMMY_NODE_ID,
                            kind: ExprKind::Path(
                                None,
                                ast::Path {
                                    span: DUMMY_SP,
                                    segments: thin_vec![
                                        PathSegment {
                                            id: DUMMY_NODE_ID,
                                            ident: Ident::from_str("std"),
                                            args: None
                                        },
                                        PathSegment {
                                            id: DUMMY_NODE_ID,
                                            ident: Ident::from_str("ptr"),
                                            args: None
                                        },
                                        PathSegment {
                                            id: DUMMY_NODE_ID,
                                            ident: Ident::from_str("null_mut")
                                        }
                                    ],
                                    tokens: None,
                                },
                            ),
                            span: DUMMY_SP,
                            attrs: thin_vec![],
                            tokens: None,
                        }),
                        thin_vec![P(array_expr)],
                    ),
                    span: DUMMY_SP,
                    attrs: thin_vec![],
                    tokens: None,
                }),
                is_shorthand: false,
                is_placeholder: false,
            };

            fields.push(expr_field);
            let expn_id = LocalExpnId::fresh_empty();
            let fragment = AstFragment::ExprFields(fields); // ::Expr()
            this.resolver.visit_ast_fragment_with_placeholders(expn_id, &fragment);
        }
        _ => {}
    }
}

impl<'a> MutVisitor for AstMutVisitor<'a> {
    fn flat_map_item(&mut self, mut item: P<Item>) -> SmallVec<[P<Item>; 1]> {
        let Item { ident: _, attrs: _, id, kind, vis: _, span: _, tokens: _ } = item.deref_mut();
        match kind {
            ItemKind::Struct(vdata, _) => {
                if self.boxable_structs.contains(id) {
                    if let VariantData::Struct(fields, _) = vdata {
                        let field_id = self.resolver.next_node_id();
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
            }
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
*/

pub fn wrap_extern_calls(
    krate: &mut ast::Crate,
    sess: &Session,
    resolver: &mut dyn ResolverExpand,
    crate_name: String,
    features: &Features,
) {
    struct ExternCallVisitor<'a> {
        extern_calls: FxHashSet<NodeId>,
        ext_cx: ExtCtxt<'a>,
    }

    impl<'a> MutVisitor for ExternCallVisitor<'a> {
        fn visit_expr(&mut self, expr: &mut P<Expr>) {
            let id = expr.id;
            if self.extern_calls.contains(&id) {
                let local_expr = expr.clone();
                let closure = self.ext_cx.expr(
                    expr.span,
                    ExprKind::Closure(Box::new(Closure {
                        binder: ast::ClosureBinder::NotPresent,
                        capture_clause: ast::CaptureBy::Ref,
                        constness: ast::Const::No,
                        asyncness: ast::Async::No,
                        movability: ast::Movability::Movable,
                        fn_decl: P(FnDecl {
                            inputs: ThinVec::new(),
                            output: ast::FnRetTy::Default(DUMMY_SP),
                        }),
                        body: local_expr,
                        fn_decl_span: expr.span,
                        fn_arg_span: expr.span,
                    })),
                );
                let wrapper_fn = self.ext_cx.path(
                    expr.span,
                    vec![
                        Ident::from_str("std"),
                        Ident::from_str("metasafe"),
                        Ident::with_dummy_span(sym::metasafe_extern_stack_run),
                    ],
                );
                let expr_path = self.ext_cx.expr_path(wrapper_fn);
                let wrapper_call = self.ext_cx.expr_call(expr.span, expr_path, thin_vec![closure]);
                let wrapper_call_frag = AstFragment::Expr(wrapper_call);
                let wrapper_call =
                    self.ext_cx.expander().fully_expand_fragment(wrapper_call_frag).make_expr();
                let _ = std::mem::replace(expr, wrapper_call);
            }
        }
    }

    let extern_calls = load_extern_calls(crate_name.clone());
    if extern_calls.is_empty() {
        return;
    }

    let econfig = ExpansionConfig::default(crate_name, features);
    let ext_cx = ExtCtxt::new(&sess, econfig, resolver, None);
    let mut extern_call_visitor = ExternCallVisitor { extern_calls, ext_cx };
    extern_call_visitor.visit_crate(krate);
}

