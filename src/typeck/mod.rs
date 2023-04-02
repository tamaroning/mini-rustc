use crate::ast::{self, BinOp, Crate, ExprKind, Ident, LetStmt, StmtKind};
use crate::middle::ty::{AdtDef, Ty};
use crate::middle::Ctxt;
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
    scopes: Vec<HashMap<&'chk String, Rc<Ty>>>,
    current_return_type: Option<&'chk Ty>,
    errors: Vec<String>,
}

impl<'ctx, 'chk: 'ctx> TypeChecker<'ctx> {
    fn new(ctx: &'ctx mut Ctxt) -> Self {
        TypeChecker {
            ctx,
            scopes: vec![],
            current_return_type: None,
            errors: vec![],
        }
    }

    fn error(&mut self, e: String) {
        self.errors.push(e);
    }

    fn insert_symbol_type(&mut self, symbol: &'chk String, ty: Rc<Ty>) {
        self.scopes
            .last_mut()
            .unwrap()
            .insert(symbol, Rc::clone(&ty));
    }

    fn get_symbol_type(&mut self, ident: &Ident) -> Option<Rc<Ty>> {
        for scope in self.scopes.iter().rev() {
            let ty = scope.get(&ident.symbol).map(Rc::clone);
            if ty.is_some() {
                return ty;
            }
        }
        None
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

    fn push_symbol_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_symbol_scope(&mut self) {
        self.scopes.pop();
    }
}

impl<'ctx> ast::visitor::Visitor<'ctx> for TypeChecker<'ctx> {
    fn visit_crate(&mut self, _krate: &'ctx Crate) {
        self.push_symbol_scope();
    }

    fn visit_crate_post(&mut self, _krate: &'ctx Crate) {
        self.pop_symbol_scope();
    }

    // TODO: allow func call before finding declaration of the func
    // TODO: what if typechecker does not find a body of non-external func?
    // TODO: external func must not have its body (correct?)
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
        self.insert_symbol_type(&func.name.symbol, func_ty);

        // push scope
        self.push_symbol_scope();
        for (param, param_ty) in &func.params {
            self.insert_symbol_type(&param.symbol, Rc::clone(param_ty));
        }
        // push return type
        self.push_return_type(&func.ret_ty);
    }
    fn visit_func_post(&mut self, func: &'ctx ast::Func) {
        if let Some(body) = &func.body {
            let body_ty = self.ctx.get_block_type(body);
            let expected = self.peek_return_type();
            if !body_ty.is_never() && &*body_ty != expected {
                self.error(format!(
                    "Expected type {:?} for func body, but found {:?}",
                    expected, body_ty
                ));
            }
        }
        // pop scope
        self.pop_symbol_scope();
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

    fn visit_block(&mut self, _block: &'ctx ast::Block) {
        self.push_symbol_scope();
    }

    fn visit_block_post(&mut self, _block: &'ctx ast::Block) {
        self.pop_symbol_scope();
    }

    fn visit_stmt_post(&mut self, stmt: &'ctx ast::Stmt) {
        let ty: Rc<Ty> = match &stmt.kind {
            StmtKind::Semi(expr) => {
                let expr_ty = self.ctx.get_type(expr.id);
                if expr_ty.is_never() {
                    Rc::new(Ty::Never)
                } else {
                    Rc::new(Ty::Unit)
                }
            }
            StmtKind::Let(LetStmt { init, .. }) => {
                if let Some(init) = init {
                    let init_ty = self.ctx.get_type(init.id);
                    if init_ty.is_never() {
                        Rc::new(Ty::Never)
                    } else {
                        Rc::new(Ty::Unit)
                    }
                } else {
                    Rc::new(Ty::Unit)
                }
            }
            StmtKind::Expr(expr) => self.ctx.get_type(expr.id),
        };
        self.ctx.insert_type(stmt.id, ty);
    }

    // TODO: handling local variables properly
    // TODO: shadowing
    fn visit_let_stmt(&mut self, let_stmt: &'ctx LetStmt) {
        // set local variable type
        self.insert_symbol_type(&let_stmt.ident.symbol, Rc::clone(&let_stmt.ty));
    }

    fn visit_let_stmt_post(&mut self, let_stmt: &'ctx LetStmt) {
        // checks if type of initalizer matches with the annotated type
        if let Some(init) = &let_stmt.init {
            let init_ty = self.ctx.get_type(init.id);
            if !init_ty.is_never() && let_stmt.ty != init_ty {
                self.error(format!(
                    "Expected `{:?}` type, but found `{:?}`",
                    let_stmt.ty, init_ty
                ));
            }
        }
    }

    // use post order
    fn visit_expr_post(&mut self, expr: &'ctx ast::Expr) {
        let ty: Rc<Ty> = match &expr.kind {
            ExprKind::NumLit(_) => Rc::new(Ty::I32),
            ExprKind::BoolLit(_) => Rc::new(Ty::Bool),
            ExprKind::StrLit(_) => Rc::new(Ty::Ref("static".to_string(), Rc::new(Ty::Str))),
            ExprKind::Unit => Rc::new(Ty::Unit),
            ExprKind::Assign(l, r) => {
                let lhs_ty = &self.ctx.get_type(l.id);
                let rhs_ty = &self.ctx.get_type(r.id);
                if rhs_ty.is_never() || **lhs_ty == **rhs_ty {
                    Rc::new(Ty::Unit)
                } else {
                    self.error(format!("Cannot assign {:?} to {:?}", rhs_ty, lhs_ty));
                    Rc::new(Ty::Error)
                }
            }
            // TODO: deal with never type
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
            // TODO: deal with never type
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
                    match self.get_symbol_type(ident) {
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
            // TODO: deal with never type params
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
            ExprKind::Block(block) => self.ctx.get_block_type(block),
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
