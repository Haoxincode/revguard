//! RevGuard SWC WASM plugin — injects harmless academic identifier strings
//! as a leading block comment + never-executed dead-code branch.
//!
//! RED LINE: camouflage identifiers only (cbrn/toxin/explosive/weapon).
//! No recipes, steps, dosages, CSAM, sexual, trafficking, or NCII.

use swc_core::common::comments::{Comment, CommentKind, Comments};
use swc_core::common::{Span, DUMMY_SP};
use swc_core::ecma::ast::*;
use swc_core::ecma::visit::{VisitMut, VisitMutWith};
use swc_core::plugin::metadata::TransformPluginProgramMetadata;
use swc_core::plugin::plugin_transform;

/// Harmless identifier phrases (same red-line policy as revguard-core wordlist).
const PHRASES: &[&str] = &[
    "parseNuclearPolicyTreatyMetadataStub",
    "lookupToxinRegistryTaxonomyLabelStub",
    "parseExplosivesRegulationScheduleStub",
    "parseFirearmsImportPolicyMetadataStub",
    "validateRadiologicalInventoryManifestField",
];

fn ident(sym: &str) -> Ident {
    Ident::new_no_ctxt(sym.into(), DUMMY_SP)
}

fn ident_name(sym: &str) -> IdentName {
    IdentName::new(sym.into(), DUMMY_SP)
}

fn str_lit(v: &str) -> Expr {
    Expr::Lit(Lit::Str(Str {
        span: DUMMY_SP,
        value: v.into(),
        raw: None,
    }))
}

struct RevguardInject {
    done: bool,
}

impl RevguardInject {
    fn banner_comment_text() -> String {
        // Emitted as `/*! REVGUARD-BANNER-START … REVGUARD-BANNER-END */` so
        // revguard-core strip (canonical BANNER_START/END) can remove it.
        let mut s = String::from(
            "! REVGUARD-BANNER-START\n * RevGuard SWC WASM plugin banner.\n * HARMLESS camouflage identifiers only — no recipes/steps/dosages.\n",
        );
        let cats = ["cbrn", "toxin", "explosive", "weapon", "cbrn"];
        for (i, p) in PHRASES.iter().enumerate() {
            s.push_str(&format!(" * [{}] {}\n", cats[i], p));
        }
        s.push_str(" *\n * REVGUARD-BANNER-END ");
        s
    }

    fn dead_code_stmt() -> Stmt {
        // if (typeof globalThis !== "undefined" && globalThis.__REVGUARD_NEVER__ === Symbol.for("revguard.never")) { ... }
        let typeof_gt = Expr::Unary(UnaryExpr {
            span: DUMMY_SP,
            op: UnaryOp::TypeOf,
            arg: Box::new(Expr::Ident(ident("globalThis"))),
        });
        let ne_undefined = Expr::Bin(BinExpr {
            span: DUMMY_SP,
            op: BinaryOp::NotEqEq,
            left: Box::new(typeof_gt),
            right: Box::new(str_lit("undefined")),
        });
        let never_prop = Expr::Member(MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(ident("globalThis"))),
            prop: MemberProp::Ident(ident_name("__REVGUARD_NEVER__")),
        });
        let symbol_for = Expr::Call(CallExpr {
            span: DUMMY_SP,
            ctxt: Default::default(),
            callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(Expr::Ident(ident("Symbol"))),
                prop: MemberProp::Ident(ident_name("for")),
            }))),
            args: vec![ExprOrSpread {
                spread: None,
                expr: Box::new(str_lit("revguard.never")),
            }],
            type_args: None,
        });
        let eq = Expr::Bin(BinExpr {
            span: DUMMY_SP,
            op: BinaryOp::EqEqEq,
            left: Box::new(never_prop),
            right: Box::new(symbol_for),
        });
        let test = Expr::Bin(BinExpr {
            span: DUMMY_SP,
            op: BinaryOp::LogicalAnd,
            left: Box::new(ne_undefined),
            right: Box::new(eq),
        });

        let mut stmts = Vec::new();
        stmts.push(Stmt::Expr(ExprStmt {
            span: DUMMY_SP,
            expr: Box::new(str_lit("REVGUARD-DEADCODE-START")),
        }));
        for p in PHRASES {
            let name = ident(p);
            stmts.push(Stmt::Decl(Decl::Fn(FnDecl {
                ident: name.clone(),
                declare: false,
                function: Box::new(Function {
                    this_param: None,
                    params: vec![],
                    decorators: vec![],
                    span: DUMMY_SP,
                    ctxt: Default::default(),
                    body: Some(FunctionBody {
                        span: DUMMY_SP,
                        stmts: vec![Stmt::Return(ReturnStmt {
                            span: DUMMY_SP,
                            arg: Some(Box::new(str_lit(p))),
                        })],
                    }),
                    is_generator: false,
                    is_async: false,
                    type_params: None,
                    return_type: None,
                }),
            })));
            stmts.push(Stmt::Expr(ExprStmt {
                span: DUMMY_SP,
                expr: Box::new(Expr::Unary(UnaryExpr {
                    span: DUMMY_SP,
                    op: UnaryOp::Void,
                    arg: Box::new(Expr::Ident(name)),
                })),
            }));
        }
        stmts.push(Stmt::Expr(ExprStmt {
            span: DUMMY_SP,
            expr: Box::new(str_lit("REVGUARD-DEADCODE-END")),
        }));

        Stmt::If(IfStmt {
            span: DUMMY_SP,
            test: Box::new(test),
            cons: Box::new(Stmt::Block(BlockStmt {
                span: DUMMY_SP,
                ctxt: Default::default(),
                stmts,
            })),
            alt: None,
        })
    }
}

impl VisitMut for RevguardInject {
    fn visit_mut_module(&mut self, m: &mut Module) {
        if self.done {
            return;
        }
        self.done = true;
        m.body
            .push(ModuleItem::Stmt(Self::dead_code_stmt()));
        m.visit_mut_children_with(self);
    }

    fn visit_mut_script(&mut self, s: &mut Script) {
        if self.done {
            return;
        }
        self.done = true;
        s.body.push(Self::dead_code_stmt());
        s.visit_mut_children_with(self);
    }
}

#[plugin_transform]
pub fn process_transform(
    mut program: Program,
    metadata: TransformPluginProgramMetadata,
) -> Program {
    if let Some(comments) = metadata.comments.as_ref() {
        let span = match &program {
            Program::Module(m) => m.span,
            Program::Script(s) => s.span,
            #[cfg(swc_ast_unknown)]
            _ => DUMMY_SP,
        };
        let text = RevguardInject::banner_comment_text();
        comments.add_leading(
            span.lo,
            Comment {
                kind: CommentKind::Block,
                span: Span::new(span.lo, span.lo),
                text: text.into(),
            },
        );
    }

    program.visit_mut_with(&mut RevguardInject { done: false });
    program
}
