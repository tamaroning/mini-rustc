use crate::analysis::Ctxt;
use crate::ast::{self, BinOp, Crate, ExprKind, Ident, LetStmt, StmtKind};
use crate::ty::{AdtDef, Ty};
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
    ctx: &'chk mut Ctxt,
    // TODO: use stacks respresenting the current scope
    /// local variables, paramters to type mappings
    ident_ty_mappings: HashMap<&'chk String, Rc<Ty>>,
    current_return_type: Option<&'chk Ty>,
    errors: Vec<String>,
}

impl<'ctx, 'chk: 'ctx> TypeChecker<'ctx> {
    fn new(ctx: &'ctx mut Ctxt) -> Self {
        TypeChecker {
            ctx,
            ident_ty_mappings: HashMap::new(),
            current_return_type: None,
            errors: vec![],
        }
    }

    fn error(&mut self, e: String) {
        self.errors.push(e);
    }

    fn insert_ident_type(&mut self, symbol: &'chk String, ty: Rc<Ty>) {
        self.ident_ty_mappings.insert(symbol, Rc::clone(&ty));
    }

    fn get_ident_type(&mut self, ident: &Ident) -> Option<Rc<Ty>> {
        self.ident_ty_mappings.get(&ident.symbol).map(Rc::clone)
    }

    fn peek_return_type(&self) -> &Ty {
        self.current_return_type.as_ref().unwrap()
    }

    fn push_return_type(&mut self, ty: &'chk Ty) {
        self.current_return_type = Some(ty);
    }

    fn pop_return_type(&mut self) {
        self.current_return_type = None;
    }
}

impl<'ctx> ast::visitor::Visitor<'ctx> for TypeChecker<'ctx> {
    // TODO: allow func call before finding declaration of the func
    // TODO: what if typechecker does not find a body of non-external func?
    // TODO: external func must not have its body (correct?)
    // TODO: typecheck func body
    // TODO: handle never type properly
    fn visit_func(&mut self, func: &'ctx ast::Func) {
        // TODO: typecheck main func
        let param_tys = func
            .params
            .iter()
            .map(|(_ident, ty)| Rc::clone(ty))
            .collect();
        let func_ty = Rc::new(Ty::Fn(param_tys, Rc::clone(&func.ret_ty)));

        self.ctx
            .set_fn_type(func.name.symbol.clone(), Rc::clone(&func_ty));
        self.insert_ident_type(&func.name.symbol, func_ty);

        for (param, param_ty) in &func.params {
            self.insert_ident_type(&param.symbol, Rc::clone(param_ty));
        }
        // set return type
        self.push_return_type(&func.ret_ty);
    }
    fn visit_func_post(&mut self, _func: &'ctx ast::Func) {
        // pop return type
        self.pop_return_type();
    }

    fn visit_struct_item(&mut self, strct: &'ctx ast::StructItem) {
        let adt = AdtDef {
            fields: strct
                .fields
                .iter()
                .map(|(s, ty)| (s.symbol.clone(), Rc::clone(ty)))
                .collect(),
        };
        self.ctx.set_adt_def(strct.ident.symbol.clone(), adt);
    }

    fn visit_stmt_post(&mut self, stmt: &'ctx ast::Stmt) {
        let ty: Rc<Ty> = match &stmt.kind {
            StmtKind::Semi(_) | StmtKind::Let(_) => Rc::new(Ty::Unit),
            StmtKind::Expr(expr) => self.ctx.get_type(expr.id),
        };
        self.ctx.insert_type(stmt.id, ty);
    }

    // TODO: handling local variables properly
    // TODO: shadowing
    fn visit_let_stmt(&mut self, let_stmt: &'ctx LetStmt) {
        // set local variable type
        self.insert_ident_type(&let_stmt.ident.symbol, Rc::clone(&let_stmt.ty));
    }

    fn visit_let_stmt_post(&mut self, let_stmt: &'ctx LetStmt) {
        // checks if type of initalizer matches with the annotated type
        if let Some(init) = &let_stmt.init {
            let init_ty = self.ctx.get_type(init.id);
            if let_stmt.ty != init_ty {
                self.error(format!(
                    "Expected `{:?}` type, but found `{:?}`",
                    let_stmt.ty, init_ty
                ));
            }
        }
    }

    // use post order
    // TODO: deal with never type
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
            ExprKind::StrLit(_) => Rc::new(Ty::Ref("static".to_string(), Rc::new(Ty::Str))),
            ExprKind::Unary(_op, inner) => {
                let inner_ty = &self.ctx.get_type(inner.id);
                if **inner_ty == Ty::I32 {
                    Rc::new(Ty::I32)
                } else {
                    self.error("inner expr of unary must be type of i32".to_string());
                    Rc::new(Ty::Error)
                }
            }
            ExprKind::Ident(ident) => {
                // lookup function name at first
                if let Some(ty) = self.ctx.lookup_fn_type(&ident.symbol) {
                    ty
                }
                // then find symbols in local variables and in parameters
                else {
                    match self.get_ident_type(ident) {
                        // TODO: lookup functions
                        Some(ty) => ty,
                        None => {
                            self.error(format!("Could not find type of {}", ident.symbol));
                            Rc::new(Ty::Error)
                        }
                    }
                }
            }
            ExprKind::Return(expr) => {
                let actual_ret_ty = self.ctx.get_type(expr.id);
                let expected_ret_ty = self.peek_return_type();
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
            ExprKind::Call(expr, args) => {
                let maybe_func_ty = self.ctx.get_type(expr.id);
                if let Ty::Fn(param_ty, ret_ty) = &*maybe_func_ty {
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
                            Rc::new(Ty::Error)
                        }
                    } else {
                        self.error(format!(
                            "Expected {} arguments, but found {}",
                            param_ty.len(),
                            args.len()
                        ));
                        Rc::new(Ty::Error)
                    }
                } else {
                    self.error(format!("Expected fn type, but found {:?}", maybe_func_ty));
                    Rc::new(Ty::Error)
                }
            }
            ExprKind::Block(block) => {
                if let Some(stmt) = block.stmts.last() {
                    let last_stmt_ty = &self.ctx.get_type(stmt.id);
                    Rc::clone(last_stmt_ty)
                } else {
                    // no statement. Unit type
                    Rc::new(Ty::Unit)
                }
            }
            ExprKind::If(_cond, then, _els) => {
                // TODO: typecheck cond and els
                self.ctx.get_type(then.id)
            }
            ExprKind::Index(array, _index) => {
                let maybe_array_ty = self.ctx.get_type(array.id);
                // TODO: typecheck index
                if let Ty::Array(elem_ty, _) = &*maybe_array_ty {
                    Rc::clone(elem_ty)
                } else {
                    self.error(format!("type {:?} cannot be indexed", maybe_array_ty));
                    Rc::new(Ty::Error)
                }
            }
            ExprKind::Field(receiver, field) => {
                let maybe_adt = self.ctx.get_type(receiver.id);
                if let Some(adt_name) = maybe_adt.get_adt_name() {
                    if let Some(adt) = self.ctx.lookup_adt_def(adt_name) {
                        let r = adt.fields.iter().find(|(f, _)| field.symbol == *f);
                        if let Some((_, ty)) = r {
                            Rc::clone(ty)
                        } else {
                            self.error(format!(
                                "Type {} does not have field `{}`",
                                adt_name, field.symbol
                            ));
                            Rc::new(Ty::Error)
                        }
                    } else {
                        self.error(format!("Cannot find type {}", adt_name));
                        Rc::new(Ty::Error)
                    }
                } else {
                    self.error("field access can used only for ADT".to_string());
                    Rc::new(Ty::Error)
                }
            }
            ExprKind::Struct(ident, _fds) => {
                let adt_name = &ident.symbol;
                if let Some(_adt) = self.ctx.lookup_adt_def(adt_name) {
                    // TODO: typecheck fields
                    Rc::new(Ty::Adt(adt_name.clone()))
                } else {
                    self.error(format!("Cannot find type {}", adt_name));
                    Rc::new(Ty::Error)
                }
            }
        };
        self.ctx.insert_type(expr.id, ty);
    }
}
