use crate::ast::{self, BinOp, Crate, ExprKind, LetStmt, Stmt, StmtKind};
use crate::middle::ty::{self, AdtDef, Ty, TyKind};
use crate::middle::Ctxt;
use std::rc::Rc;

pub fn typeck<'ctx, 'chk>(
    ctx: &'chk mut Ctxt<'ctx>,
    krate: &'chk Crate,
) -> Result<(), Vec<String>> {
    let mut checker = TypeChecker::new(ctx);
    ast::visitor::go(&mut checker, krate);
    if checker.errors.is_empty() {
        Ok(())
    } else {
        Err(checker.errors)
    }
}

struct TypeChecker<'ctx, 'chk> {
    ctx: &'chk mut Ctxt<'ctx>,
    current_return_type: Option<Ty>,
    errors: Vec<String>,
}

impl<'ctx, 'chk> TypeChecker<'ctx, 'chk> {
    fn new(ctx: &'chk mut Ctxt<'ctx>) -> Self {
        TypeChecker {
            ctx,
            current_return_type: None,
            errors: vec![],
        }
    }

    fn error(&mut self, e: String) {
        self.errors.push(e);
    }

    fn peek_return_type(&self) -> &Ty {
        self.current_return_type.as_ref().unwrap()
    }

    fn push_return_type(&mut self, ty: Ty) {
        self.current_return_type = Some(ty);
    }

    fn pop_return_type(&mut self) {
        self.current_return_type = None;
    }

    fn get_block_type(&self, block: &ast::Block) -> Rc<Ty> {
        if let Some(stmt) = block.stmts.last() {
            let last_stmt_ty = &self.ctx.get_type(stmt.id);
            Rc::clone(last_stmt_ty)
        } else {
            // no statement. Unit type
            Rc::new(Ty::unit())
        }
    }
    fn ast_ty_to_ty(&mut self, ast_ty: &ast::Ty) -> self::Ty {
        let kind = match &ast_ty.kind {
            ast::TyKind::I32 => ty::TyKind::I32,
            ast::TyKind::Never => ty::TyKind::Never,
            ast::TyKind::Bool => ty::TyKind::Bool,
            ast::TyKind::Unit => ty::TyKind::Unit,
            ast::TyKind::Str => ty::TyKind::Str,
            ast::TyKind::Ref(_region, referent) => {
                ty::TyKind::Ref(Rc::new(self.ast_ty_to_ty(&referent)))
            }
            ast::TyKind::Array(elem_ty, n) => {
                ty::TyKind::Array(Rc::new(self.ast_ty_to_ty(elem_ty)), *n)
            }
            ast::TyKind::Adt(path) => {
                if let Some(binding) = self.ctx.resolve_path(path) {
                    ty::TyKind::Adt(Rc::clone(&binding.cpath))
                } else {
                    self.error(format!("{:?}", path));
                    ty::TyKind::Error
                }
            }
        };
        Ty::new(kind)
    }
}

impl<'chk> ast::visitor::Visitor<'chk> for TypeChecker<'_, 'chk> {
    fn visit_crate(&mut self, _krate: &'chk Crate) {}

    fn visit_crate_post(&mut self, _krate: &'chk Crate) {}

    // TODO: allow func call before finding declaration of the func
    // TODO: what if typechecker does not find a body of non-external func?
    // TODO: external func must not have its body (correct?)
    fn visit_func(&mut self, func: &'chk ast::Func) {
        // TODO: typecheck main func
        let param_tys = func
            .params
            .iter()
            .map(|(_ident, ty)| Rc::new(self.ast_ty_to_ty(ty)))
            .collect();
        let func_ty = Rc::new(Ty::new(TyKind::Fn(
            Rc::new(param_tys),
            Rc::new(self.ast_ty_to_ty(&func.ret_ty)),
        )));

        let binding = self.ctx.get_binding(&func.name).unwrap();
        self.ctx.set_name_type(Rc::clone(&binding), func_ty);

        // push scope
        for (param, param_ty) in &func.params {
            let binding = self.ctx.get_binding(param).unwrap();
            let param_ty = self.ast_ty_to_ty(param_ty);
            self.ctx
                .set_name_type(Rc::clone(&binding), Rc::new(param_ty));
        }
        // push return type
        let ret_ty = self.ast_ty_to_ty(&func.ret_ty);
        self.push_return_type(ret_ty);
    }

    fn visit_func_post(&mut self, func: &'chk ast::Func) {
        let Some(body) = &func.body else {
            return;
        };

        let body_ty = self.ctx.get_type(body.id);

        let expected = self.peek_return_type();
        if !body_ty.is_never() && *body_ty != *expected {
            self.error(format!(
                "Expected type {:?} for func body, but found {:?}",
                expected, body_ty
            ));
        }
        // pop return type
        self.pop_return_type();
    }

    fn visit_struct_item(&mut self, strct: &'chk ast::StructItem) {
        let field_tys: Vec<(Rc<String>, Rc<Ty>)> = strct
            .fields
            .iter()
            .map(|(name, ty)| (Rc::clone(&name.symbol), Rc::new(self.ast_ty_to_ty(ty))))
            .collect();
        let adt = AdtDef { fields: field_tys };
        let binding = self.ctx.get_binding(&strct.ident).unwrap();
        self.ctx.set_adt_def(Rc::clone(&binding.cpath), adt);
    }

    fn visit_stmt_post(&mut self, stmt: &'chk ast::Stmt) {
        let ty: Rc<Ty> = match &stmt.kind {
            StmtKind::Semi(expr) => {
                let expr_ty = self.ctx.get_type(expr.id);
                if expr_ty.is_never() {
                    Rc::new(Ty::never())
                } else {
                    Rc::new(Ty::unit())
                }
            }
            StmtKind::Let(LetStmt { init, ty, ident: _ }) => {
                if let Some(init) = init {
                    let init_ty = self.ctx.get_type(init.id);
                    let annotated_ty = self.ast_ty_to_ty(ty.as_ref().unwrap());
                    if init_ty.is_never() {
                        Rc::new(Ty::never())
                    } else {
                        if annotated_ty != *init_ty {
                            self.error(format!(
                                "Expected `{:?}` type, but found `{:?}`",
                                annotated_ty, init_ty
                            ));
                            Rc::new(Ty::error())
                        } else {
                            Rc::new(Ty::unit())
                        }
                    }
                } else {
                    Rc::new(Ty::unit())
                }
            }
            StmtKind::Expr(expr) => self.ctx.get_type(expr.id),
        };
        self.ctx.insert_type(stmt.id, ty);
    }

    // TODO: handling local variables properly
    // TODO: shadowing
    fn visit_stmt(&mut self, stmt: &'chk Stmt) {
        match &stmt.kind {
            StmtKind::Let(let_stmt) => {
                // set local variable type
                let binding = self.ctx.get_binding(&let_stmt.ident).unwrap();
                // set type of local variable
                // TODO: unwrap
                let annotated_ty = self.ast_ty_to_ty(let_stmt.ty.as_ref().unwrap());
                self.ctx
                    .set_name_type(Rc::clone(&binding), Rc::new(annotated_ty));
                // set type of statement
                let stmt_ty = self.ast_ty_to_ty(let_stmt.ty.as_ref().unwrap());
                // TODO: unwrap
                self.ctx.insert_type(stmt.id, Rc::new(stmt_ty));
            }
            _ => {}
        }
    }

    // use post order
    fn visit_expr_post(&mut self, expr: &'chk ast::Expr) {
        let ty: Rc<Ty> = match &expr.kind {
            ExprKind::NumLit(_) => Rc::new(Ty::new(TyKind::I32)),
            ExprKind::BoolLit(_) => Rc::new(Ty::new(TyKind::Bool)),
            ExprKind::StrLit(_) => Rc::new(Ty::new(TyKind::Ref(Rc::new(Ty::new(TyKind::Str))))),
            ExprKind::Unit => Rc::new(Ty::unit()),
            ExprKind::Assign(l, r) => {
                let lhs_ty = &self.ctx.get_type(l.id);
                let rhs_ty = &self.ctx.get_type(r.id);
                if rhs_ty.is_never() || **lhs_ty == **rhs_ty {
                    Rc::new(Ty::unit())
                } else {
                    self.error(format!("Cannot assign {:?} to {:?}", rhs_ty, lhs_ty));
                    Rc::new(Ty::error())
                }
            }
            // TODO: deal with never type
            ExprKind::Binary(op, l, r) => {
                let lhs_ty = &self.ctx.get_type(l.id);
                let rhs_ty = &self.ctx.get_type(r.id);
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul => {
                        if lhs_ty.kind == TyKind::I32 && rhs_ty.kind == TyKind::I32 {
                            Rc::new(Ty::new(TyKind::I32))
                        } else {
                            self.error("Both lhs and rhs must be type of i32".to_string());
                            Rc::new(Ty::error())
                        }
                    }
                    BinOp::Gt | BinOp::Lt => {
                        if lhs_ty.kind == TyKind::I32 && rhs_ty.kind == TyKind::I32 {
                            Rc::new(Ty::new(TyKind::I32))
                        } else {
                            self.error("Both lhs and rhs must be type of i32".to_string());
                            Rc::new(Ty::error())
                        }
                    }
                    BinOp::Eq | BinOp::Ne => {
                        // TODO: other types?
                        if (lhs_ty.kind == TyKind::I32 && rhs_ty.kind == TyKind::I32)
                            || (lhs_ty.kind == TyKind::Bool && rhs_ty.kind == TyKind::Bool)
                        {
                            Rc::new(Ty::new(TyKind::Bool))
                        } else {
                            self.error("Both lhs and rhs must have the same type".to_string());
                            Rc::new(Ty::error())
                        }
                    }
                }
            }
            // TODO: deal with never type
            ExprKind::Unary(_op, inner) => {
                let inner_ty = &self.ctx.get_type(inner.id);
                if inner_ty.kind == TyKind::I32 {
                    Rc::new(Ty::new(TyKind::I32))
                } else {
                    self.error("inner expr of unary must be type of i32".to_string());
                    Rc::new(Ty::error())
                }
            }
            ExprKind::Path(path) => {
                // find symbols in local variables, parameters, and in functions
                if let Some(binding) = self.ctx.resolve_path(path) {
                    if let Some(ty) = self.ctx.lookup_name_type(&binding) {
                        ty
                    } else {
                        self.error(format!("Cannot use `{:?}` before declaration", path));
                        Rc::new(Ty::error())
                    }
                } else {
                    self.error(format!("Could not resolve ident `{:?}`", path));
                    Rc::new(Ty::error())
                }
            }

            ExprKind::Return(expr) => {
                let actual_ret_ty = self.ctx.get_type(expr.id);
                let expected_ret_ty = self.peek_return_type();
                if *actual_ret_ty == *expected_ret_ty {
                    Rc::new(Ty::never())
                } else {
                    self.error(format!(
                        "Expected {:?} type, but {:?} returned",
                        expected_ret_ty, actual_ret_ty
                    ));
                    Rc::new(Ty::error())
                }
            }
            // TODO: deal with never type params
            ExprKind::Call(expr, args) => {
                let maybe_func_ty = self.ctx.get_type(expr.id);
                if let TyKind::Fn(param_ty, ret_ty) = &maybe_func_ty.kind {
                    if param_ty.len() == args.len() {
                        let mut ok = true;
                        for (arg, param_ty) in args.iter().zip(param_ty.iter()) {
                            let arg_ty = &self.ctx.get_type(arg.id);
                            if arg_ty != param_ty {
                                self.error(format!(
                                    "Expected {:?} type argument, but found {:?} type",
                                    param_ty, arg_ty
                                ));
                                ok = false;
                            }
                        }
                        if ok {
                            Rc::clone(ret_ty)
                        } else {
                            Rc::new(Ty::error())
                        }
                    } else {
                        self.error(format!(
                            "Expected {} arguments, but found {}",
                            param_ty.len(),
                            args.len()
                        ));
                        Rc::new(Ty::error())
                    }
                } else {
                    self.error(format!("Expected fn type, but found {:?}", maybe_func_ty));
                    Rc::new(Ty::error())
                }
            }
            ExprKind::Block(block) => self.ctx.get_type(block.id),
            ExprKind::If(cond, then, els) => {
                let cond_ty = self.ctx.get_type(cond.id);
                let then_ty = self.ctx.get_type(then.id);
                if cond_ty.is_never() || cond_ty.kind == TyKind::Bool {
                    let els_ty = if let Some(els) = els {
                        self.ctx.get_type(els.id)
                    } else {
                        Rc::new(Ty::unit())
                    };

                    if then_ty.is_never() || then_ty.kind == els_ty.kind {
                        then_ty
                    } else {
                        self.error(format!(
                            "Type mismatch then block has `{:?}`, but else block has `{:?}`",
                            then_ty, els_ty
                        ));
                        Rc::new(Ty::error())
                    }
                } else {
                    self.error(format!(
                        "Expected bool for conditional, but found {:?}",
                        cond_ty
                    ));
                    Rc::new(Ty::error())
                }
            }
            ExprKind::Index(array, _index) => {
                let maybe_array_ty = self.ctx.get_type(array.id);
                // TODO: typecheck index
                if let TyKind::Array(elem_ty, _) = &maybe_array_ty.kind {
                    Rc::clone(elem_ty)
                } else {
                    self.error(format!("type {:?} cannot be indexed", maybe_array_ty));
                    Rc::new(Ty::error())
                }
            }
            ExprKind::Field(receiver, field) => {
                let maybe_adt = self.ctx.get_type(receiver.id);
                if let Some(cpath) = maybe_adt.get_adt_name() {
                    if let Some(adt) = self.ctx.lookup_adt_def(cpath) {
                        let r = adt.fields.iter().find(|(f, _)| field.symbol == *f);
                        if let Some((_, ty)) = r {
                            Rc::clone(ty)
                        } else {
                            self.error(format!(
                                "Type {:?} does not have field `{}`",
                                cpath, field.symbol
                            ));
                            Rc::new(Ty::error())
                        }
                    } else {
                        self.error(format!("receiver is not struct, but {:?}", maybe_adt));
                        Rc::new(Ty::error())
                    }
                } else {
                    self.error("field access can used only for ADT".to_string());
                    Rc::new(Ty::error())
                }
            }
            ExprKind::Struct(path, _fds) => {
                if let Some(binding) = self.ctx.resolve_path(path) {
                    if let Some(_adt) = self.ctx.lookup_adt_def(&binding.cpath) {
                        // TODO: typecheck fields
                        Rc::new(Ty::new(TyKind::Adt(Rc::clone(&binding.cpath))))
                    } else {
                        self.error(format!("{:?} does not have struct type", binding.cpath));
                        Rc::new(Ty::error())
                    }
                } else {
                    self.error(format!("Could not resolve {}", path.span.to_snippet()));
                    Rc::new(Ty::error())
                }
            }
            ExprKind::Array(elems) => {
                if elems.is_empty() {
                    // TODO: type inference: typecheck arary with zero element
                    self.error("Array with zero element is not supported".to_string());
                    Rc::new(Ty::error())
                } else {
                    let first_elem = elems.first().unwrap();
                    let first_elem_ty = self.ctx.get_type(first_elem.id);

                    if first_elem_ty.is_never() {
                        self.error(format!(
                            "First element `{}` has never type. Could not infer type of array `{}`.",
                            first_elem.span.to_snippet(),
                            expr.span.to_snippet(),
                        ));
                        Rc::new(Ty::error())
                    } else {
                        let mut saw_error = false;
                        for elem in elems {
                            let elem_ty = self.ctx.get_type(elem.id);
                            if !elem_ty.is_never() && elem_ty != first_elem_ty {
                                self.error(format!(
                                    "Expected type `{:?}`, but `{}` has type `{:?}`",
                                    first_elem_ty,
                                    elem.span.to_snippet(),
                                    elem_ty,
                                ));
                                saw_error = true;
                            }
                        }
                        if saw_error {
                            Rc::new(Ty::error())
                        } else {
                            Rc::new(Ty::new(TyKind::Array(first_elem_ty, elems.len())))
                        }
                    }
                }
            }
        };
        self.ctx.insert_type(expr.id, ty);
    }

    fn visit_block_post(&mut self, block: &'chk ast::Block) {
        let block_ty = self.get_block_type(block);
        self.ctx.insert_type(block.id, block_ty);
    }
}
