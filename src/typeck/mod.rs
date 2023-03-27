use crate::analysis::Ctxt;
use crate::ast::{self, Crate, ExprKind, Ident, LetStmt};
use crate::ty::Ty;
use std::collections::HashMap;
use std::rc::Rc;

pub fn typeck(ctx: &mut Ctxt, krate: &Crate) -> Result<(), Vec<String>> {
    let mut checker = TypeChecker::new(ctx);
    ast::visitor::go(&mut checker, krate);
    if checker.errors.is_empty() {
        Ok(())
    } else {
        Err(checker.errors)
    }
}

struct TypeChecker<'chk> {
    local_ty_mappings: HashMap<&'chk String, Rc<Ty>>,
    ctx: &'chk mut Ctxt,
    errors: Vec<String>,
}

impl<'ctx> TypeChecker<'ctx> {
    fn new(ctx: &'ctx mut Ctxt) -> Self {
        TypeChecker {
            local_ty_mappings: HashMap::new(),
            ctx,
            errors: vec![],
        }
    }

    fn error(&mut self, e: String) {
        self.errors.push(e);
    }

    fn insert_local_type(&mut self, ident: &'ctx Ident, ty: Ty) {
        self.local_ty_mappings.insert(&ident.symbol, Rc::new(ty));
    }

    fn get_local_type(&mut self, ident: &Ident) -> Option<Rc<Ty>> {
        self.local_ty_mappings.get(&ident.symbol).map(Rc::clone)
    }
}

impl<'ctx> ast::visitor::Visitor<'ctx> for TypeChecker<'ctx> {
    fn visit_let_stmt(&mut self, let_stmt: &'ctx ast::LetStmt) {
        let LetStmt { ident } = let_stmt;
        self.insert_local_type(ident, Ty::I32);
    }

    fn visit_expr_post(&mut self, expr: &'ctx ast::Expr) {
        let ty: Rc<Ty> = match &expr.kind {
            ExprKind::Assign(l, r) => {
                let lhs_ty = &self.ctx.get_type(l.id);
                let rhs_ty = &self.ctx.get_type(r.id);
                if **lhs_ty == **rhs_ty {
                    Rc::new(Ty::Unit)
                } else {
                    self.error("lhs and rhs of assign have different types".to_string());
                    return;
                }
            }
            ExprKind::Binary(_op, l, r) => {
                let lhs_ty = &self.ctx.get_type(l.id);
                let rhs_ty = &self.ctx.get_type(r.id);
                if **lhs_ty == Ty::I32 && **rhs_ty == Ty::I32 {
                    Rc::new(Ty::I32)
                } else {
                    self.error("Both lhs and rhs must be type of i32".to_string());
                    return;
                }
            }
            ExprKind::NumLit(_) => Rc::new(Ty::I32),
            ExprKind::Unary(_op, inner) => {
                let inner_ty = &self.ctx.get_type(inner.id);
                if **inner_ty == Ty::I32 {
                    Rc::new(Ty::I32)
                } else {
                    self.error("inner expr of unary must be type of i32".to_string());
                    return;
                }
            }
            ExprKind::Ident(ident) => match self.get_local_type(ident) {
                Some(ty) => ty,
                None => {
                    self.error(format!("Could not find type of {}", ident.symbol));
                    return;
                }
            },
            ExprKind::Return(_) => Rc::new(Ty::Never),
        };
        self.ctx.insert_type(expr.id, ty);
    }
}
