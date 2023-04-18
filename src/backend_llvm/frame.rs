use super::{Codegen, LLReg, LLTy};
use crate::{
    ast::{self, StmtKind},
    middle::ty::Ty,
    resolve::{Binding, BindingKind},
    span::Ident,
};
use std::{collections::HashMap, rc::Rc};

pub fn compute_frame<'gen, 'ctx>(codegen: &mut Codegen<'gen, 'ctx>, func: &ast::Func) -> Frame {
    let mut analyzer = VisitFrame {
        codegen,
        frame: Frame::new(),
    };
    ast::visitor::go_func(&mut analyzer, func);
    analyzer.frame
}

#[derive(Debug)]
pub struct Frame {
    locals: HashMap<Rc<Binding>, Rc<Local>>,
    /// Registers pointing to memory for temporary variables
    /// Can be used only for non-lvalue array and structs
    temporary_regs: HashMap<ast::NodeId, Rc<LLReg>>,
    sret_reg: Option<Rc<LLReg>>,
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
            sret_reg: None,
            next_reg: 0,
            next_tmp_reg: 0,
        }
    }

    pub fn set_sret_reg(&mut self, reg: Rc<LLReg>) {
        self.sret_reg = Some(reg);
    }

    pub fn get_sret_reg(&self) -> Option<Rc<LLReg>> {
        self.sret_reg.as_ref().map(Rc::clone)
    }

    pub fn get_local(&self, name: &Binding) -> Rc<Local> {
        Rc::clone(self.locals.get(name).unwrap())
    }

    pub fn get_locals(&self) -> &HashMap<Rc<Binding>, Rc<Local>> {
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

pub struct VisitFrame<'ctx, 'gen, 'frm> {
    pub codegen: &'frm mut Codegen<'gen, 'ctx>,
    pub frame: Frame,
}

impl VisitFrame<'_, '_, '_> {
    fn add_local(
        &mut self,
        ident: &Ident,
        ty: &Rc<Ty>,
        binding_kind: BindingKind,
        local_kind: LocalKind,
    ) {
        // TODO: align
        let mut reg_ty = self.codegen.ty_to_llty(ty);
        if local_kind == LocalKind::Ptr {
            reg_ty = LLTy::Ptr(Rc::new(reg_ty));
        }
        let name_binding = self.codegen.ctx.get_binding(ident).unwrap();
        let reg_name_postfix = if let BindingKind::Let(shadowed_idx) = binding_kind {
            format!(".spill{}", shadowed_idx)
        } else {
            "".to_owned()
        };
        let reg_name = format!("%{}{}", ident.symbol, reg_name_postfix);
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
        let binding = self.codegen.ctx.get_binding(&func.name).unwrap();
        let (param_tys, _ret_ty) = self
            .codegen
            .ctx
            .lookup_name_type(&binding)
            .unwrap()
            .get_func_type()
            .unwrap();

        for ((param, _), param_ty) in func.params.iter().zip(param_tys.iter()) {
            if self.codegen.ty_to_llty(param_ty).eval_to_ptr() {
                // argument passed via memory (i.e. call by reference)
                self.add_local(param, param_ty, binding.kind, LocalKind::Ptr);
            } else {
                // argument passed via register (i.e. call by value)
                self.add_local(param, param_ty, binding.kind, LocalKind::Value);
            }
        }
    }

    fn visit_stmt(&mut self, stmt: &'ctx ast::Stmt) {
        match &stmt.kind {
            StmtKind::Let(let_stmt) => {
                let binding = self.codegen.ctx.get_binding(&let_stmt.ident).unwrap();
                let var_ty = self.codegen.ctx.lookup_name_type(&binding).unwrap();

                if self.codegen.ty_to_llty(&var_ty).is_void() {
                    // cannot `alloca void` so register void-like (i.e. `()`) local variables as `LocalKind::Value`
                    self.add_local(&let_stmt.ident, &var_ty, binding.kind, LocalKind::Value);
                } else {
                    self.add_local(&let_stmt.ident, &var_ty, binding.kind, LocalKind::Ptr);
                }
            }
            _ => (),
        }
    }

    fn visit_expr(&mut self, expr: &'ctx ast::Expr) {
        if matches!(
            &expr.kind,
            ast::ExprKind::Array(_) | ast::ExprKind::Struct(_, _)
        ) || (matches!(&expr.kind, ast::ExprKind::Call(_, _))
            && self
                .codegen
                .ty_to_llty(&self.codegen.ctx.get_type(expr.id))
                .eval_to_ptr())
        {
            let ty = self.codegen.ctx.get_type(expr.id);
            self.add_temporary(expr.id, &ty);
        }
    }
}
