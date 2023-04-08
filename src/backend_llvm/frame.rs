use super::{Codegen, LLReg, LLTy};
use crate::{ast, middle::ty::Ty, resolve::NameBinding};
use std::{collections::HashMap, rc::Rc};

pub fn compute_frame(codegen: &mut Codegen, func: &ast::Func) -> Frame {
    let mut frame = Frame::new();
    let mut analyzer = VisitFrame {
        codegen,
        frame: &mut frame,
    };
    ast::visitor::go_func(&mut analyzer, func);
    frame
}

#[derive(Debug)]
pub struct Frame {
    locals: HashMap<NameBinding, Rc<Local>>,
    /// Registers pointing to memory for temporary variables
    /// Can be used only for non-lvalue array and structs
    temporary_regs: HashMap<ast::NodeId, Rc<LLReg>>,
    next_reg: usize,
    next_tmp_reg: usize,
}

#[derive(Debug)]
pub struct Local {
    pub kind: LocalKind,
    pub reg: Rc<LLReg>,
}

impl Local {
    fn new(kind: LocalKind, reg: Rc<LLReg>) -> Self {
        Local { kind, reg }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LocalKind {
    /// Allocated on stack
    Ptr,
    /// Not allocated, which means the variable is passed via registers or has void-like (i.e. `()`) type
    Value,
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            locals: HashMap::new(),
            temporary_regs: HashMap::new(),
            next_reg: 1,
            next_tmp_reg: 1,
        }
    }

    pub fn get_local(&self, name: &NameBinding) -> Rc<Local> {
        Rc::clone(self.locals.get(name).unwrap())
    }

    pub fn get_locals(&self) -> &HashMap<NameBinding, Rc<Local>> {
        &self.locals
    }

    pub fn get_ptr_to_temporary(&self, node_id: ast::NodeId) -> Option<Rc<LLReg>> {
        self.temporary_regs.get(&node_id).map(Rc::clone)
    }

    pub fn get_ptrs_to_temporary(&self) -> &HashMap<ast::NodeId, Rc<LLReg>> {
        &self.temporary_regs
    }

    pub fn get_fresh_reg(&mut self) -> String {
        let i = self.next_reg;
        self.next_reg += 1;
        format!("%{i}")
    }

    fn get_fresh_tmp_reg(&mut self) -> String {
        let i = self.next_tmp_reg;
        self.next_tmp_reg += 1;
        format!("%tmp{i}")
    }
}

pub struct VisitFrame<'a, 'b, 'c> {
    pub codegen: &'a mut Codegen<'b>,
    pub frame: &'c mut Frame,
}

impl VisitFrame<'_, '_, '_> {
    fn add_local(&mut self, ident: &ast::Ident, ty: &Rc<Ty>, local_kind: LocalKind) {
        // TODO: align
        let mut reg_ty = self.codegen.ty_to_llty(ty);
        if local_kind == LocalKind::Ptr {
            reg_ty = LLTy::Ptr(Rc::new(reg_ty));
        }
        let name_binding = self.codegen.ctx.resolve_ident(ident).unwrap();
        let reg_name = format!("%{}", ident.symbol);
        let reg = LLReg::new(reg_name, Rc::new(reg_ty));
        self.frame
            .locals
            .insert(name_binding, Rc::new(Local::new(local_kind, reg)));
    }

    fn add_temporary(&mut self, node_id: ast::NodeId, ty: &Rc<Ty>) {
        // `%Struct.S` => `%Struct.S* %1`
        let llty = Rc::new(LLTy::Ptr(Rc::new(self.codegen.ty_to_llty(ty))));
        let reg_name = self.frame.get_fresh_tmp_reg();
        let reg = LLReg::new(reg_name, llty);
        self.frame.temporary_regs.insert(node_id, reg);
    }
}

impl<'ctx: 'a, 'a> ast::visitor::Visitor<'ctx> for VisitFrame<'_, '_, '_> {
    fn visit_func(&mut self, func: &'ctx ast::Func) {
        let name = self.codegen.ctx.resolve_ident(&func.name).unwrap();
        let (param_tys, _ret_ty) = self
            .codegen
            .ctx
            .lookup_name_type(&name)
            .unwrap()
            .get_func_type()
            .unwrap();

        for ((param, _), param_ty) in func.params.iter().zip(param_tys.iter()) {
            if self.codegen.ty_to_llty(param_ty).eval_to_ptr() {
                // argument passed via memory (i.e. call by reference)
                self.add_local(param, param_ty, LocalKind::Ptr);
            } else {
                // argument passed via register (i.e. call by value)
                self.add_local(param, param_ty, LocalKind::Value);
            }
        }
    }

    fn visit_let_stmt(&mut self, let_stmt: &'ctx ast::LetStmt) {
        let name = self.codegen.ctx.resolve_ident(&let_stmt.ident).unwrap();
        let var_ty = self.codegen.ctx.lookup_name_type(&name).unwrap();

        if self.codegen.ty_to_llty(&var_ty).is_void() {
            // cannot `alloca void` so register void-like (i.e. `()`) local variables as `LocalKind::Value`
            self.add_local(&let_stmt.ident, &var_ty, LocalKind::Value);
        } else {
            self.add_local(&let_stmt.ident, &var_ty, LocalKind::Ptr);
        }
    }

    fn visit_expr(&mut self, expr: &'ctx ast::Expr) {
        if matches!(
            &expr.kind,
            ast::ExprKind::Array(_) | ast::ExprKind::Struct(_, _)
        ) {
            let ty = self.codegen.ctx.get_type(expr.id);
            self.add_temporary(expr.id, &ty);
        }
    }
}
