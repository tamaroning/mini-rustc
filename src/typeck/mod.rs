use crate::analysis::Ctxt;
use crate::ast::{self, BinOp, Crate, ExprKind, Ident, LetStmt};
use crate::ty::Ty;
use std::collections::HashMap;
use std::rc::Rc;

pub fn typeck(ctx: &mut Ctxt, krate: &Crate) -> Result<(), Vec<String>> {
    let mut checker = TypeChecker::new(ctx);
    //ast::visitor::go(&mut checker, krate);
    if checker.errors.is_empty() {
        Ok(())
    } else {
        Err(checker.errors)
    }
}

struct TypeChecker<'chk> {
    local_ty_mappings: HashMap<&'chk String, Rc<Ty>>,
    current_return_type: Option<Rc<Ty>>,
    ctx: &'chk mut Ctxt,
    errors: Vec<String>,
}

impl<'ctx> TypeChecker<'ctx> {
    fn new(ctx: &'ctx mut Ctxt) -> Self {
        TypeChecker {
            local_ty_mappings: HashMap::new(),
            current_return_type: None,
            ctx,
            errors: vec![],
        }
    }

    fn error(&mut self, e: String) {
        self.errors.push(e);
    }

    fn insert_local_type(&mut self, ident: &'ctx Ident, ty: Rc<Ty>) {
        self.local_ty_mappings.insert(&ident.symbol, Rc::clone(&ty));
    }

    fn get_local_type(&mut self, ident: &Ident) -> Option<Rc<Ty>> {
        self.local_ty_mappings.get(&ident.symbol).map(Rc::clone)
    }

    fn peek_return_type(&self) -> &Rc<Ty> {
        self.current_return_type.as_ref().unwrap()
    }

    fn push_return_type(&mut self, ty: Rc<Ty>) {
        self.current_return_type = Some(ty);
    }

    fn pop_return_type(&mut self) {
        self.current_return_type = None;
    }
}

impl<'ctx> ast::visitor::Visitor<'ctx> for TypeChecker<'ctx> {
    fn visit_func(&mut self, _func: &'ctx ast::Func) {
        self.push_return_type(Rc::new(Ty::I32));
    }
    fn visit_func_post(&mut self, _func: &'ctx ast::Func) {
        self.pop_return_type();
    }

    fn visit_let_stmt(&mut self, let_stmt: &'ctx ast::LetStmt) {
        let LetStmt { ident, ty } = let_stmt;
        self.insert_local_type(ident, Rc::clone(ty));
    }

    // use post order
    fn visit_expr_post(&mut self, expr: &'ctx ast::Expr) {
        let ty: Rc<Ty> = match &expr.kind {
            ExprKind::Assign(l, r) => {
                let lhs_ty = &self.ctx.get_type(l.id);
                let rhs_ty = &self.ctx.get_type(r.id);
                if **lhs_ty == **rhs_ty {
                    Rc::new(Ty::Unit)
                } else {
                    self.error(format!("Cannot assign {:?} to {:?}", rhs_ty, lhs_ty));
                    Rc::new(Ty::Error)
                }
            }
            ExprKind::Binary(op, l, r) => {
                let lhs_ty = &self.ctx.get_type(l.id);
                let rhs_ty = &self.ctx.get_type(r.id);
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul => {
                        if **lhs_ty == Ty::I32 && **rhs_ty == Ty::I32 {
                            Rc::new(Ty::I32)
                        } else {
                            self.error("Both lhs and rhs must be type of i32".to_string());
                            Rc::new(Ty::Error)
                        }
                    }
                    BinOp::Gt | BinOp::Lt => {
                        if **lhs_ty == Ty::I32 && **rhs_ty == Ty::I32 {
                            Rc::new(Ty::Bool)
                        } else {
                            self.error("Both lhs and rhs must be type of i32".to_string());
                            Rc::new(Ty::Error)
                        }
                    }
                    BinOp::Eq | BinOp::Ne => {
                        if **lhs_ty == Ty::I32 && **rhs_ty == Ty::I32 {
                            Rc::new(Ty::Bool)
                        } else {
                            self.error("Both lhs and rhs must have the same type".to_string());
                            Rc::new(Ty::Error)
                        }
                    }
                }
            }
            ExprKind::NumLit(_) => Rc::new(Ty::I32),
            ExprKind::BoolLit(_) => Rc::new(Ty::Bool),
            ExprKind::Unary(_op, inner) => {
                let inner_ty = &self.ctx.get_type(inner.id);
                if **inner_ty == Ty::I32 {
                    Rc::new(Ty::I32)
                } else {
                    self.error("inner expr of unary must be type of i32".to_string());
                    Rc::new(Ty::Error)
                }
            }
            ExprKind::Ident(ident) => match self.get_local_type(ident) {
                Some(ty) => ty,
                None => {
                    self.error(format!("Could not find type of {}", ident.symbol));
                    Rc::new(Ty::Error)
                }
            },
            ExprKind::Return(expr) => {
                let actual_ret_ty = self.ctx.get_type(expr.id);
                let expected_ret_ty = Rc::clone(self.current_return_type.as_ref().unwrap());
                if *actual_ret_ty == *expected_ret_ty {
                    Rc::new(Ty::Never)
                } else {
                    self.error(format!(
                        "Expected {:?} type, but {:?} returned",
                        expected_ret_ty, actual_ret_ty
                    ));
                    Rc::new(Ty::Error)
                }
            }
            ExprKind::Call(ident) => {
                // TODO: name resolution is required to typecheck call expr
                Rc::new(Ty::Error)
            }
        };
        self.ctx.insert_type(expr.id, ty);
    }
}
