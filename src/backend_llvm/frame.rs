use std::{collections::HashMap, rc::Rc};
use crate::{ast, middle::ty::Ty, resolve::NameBinding};
use super::{Codegen, LLReg, LLTy};

#[derive(Debug)]
pub struct Frame {
    locals: HashMap<NameBinding, Rc<Local>>,
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
    /// Not allocated, which means the variable is passed via registers
    Value,
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            locals: HashMap::new(),
        }
    }

    pub fn get_local(&self, name: &NameBinding) -> Rc<Local> {
        Rc::clone(self.locals.get(name).unwrap())
    }

    pub fn get_locals(&self) -> &HashMap<NameBinding, Rc<Local>> {
        &self.locals
    }
}

pub struct VisitFrame<'a, 'b, 'c> {
    pub codegen: &'a mut Codegen<'b>,
    pub frame: &'c mut Frame,
}

impl VisitFrame<'_, '_, '_> {
    fn add_local(&mut self, ident: &ast::Ident, ty: &Rc<Ty>, local_kind: LocalKind) {
        // TODO: align
        // let align = self.codegen.ctx.get_align(ty);
        let mut reg_ty = self.codegen.ty_to_llty(ty);
        if local_kind == LocalKind::Ptr {
            reg_ty = LLTy::Ptr(Rc::new(reg_ty));
        }
        let name_binding = self.codegen.ctx.resolver.resolve_ident(ident).unwrap();
        let reg_name = format!("%{}", ident.symbol);
        let reg = LLReg::new(reg_name, Rc::new(reg_ty));
        self.frame
            .locals
            .insert(name_binding, Rc::new(Local::new(local_kind, reg)));
    }
}

impl<'ctx: 'a, 'a> ast::visitor::Visitor<'ctx> for VisitFrame<'_, '_, '_> {
    fn visit_func(&mut self, func: &'ctx ast::Func) {
        for (param, param_ty) in &func.params {
            self.add_local(param, param_ty, LocalKind::Value);
        }
    }

    fn visit_let_stmt(&mut self, let_stmt: &'ctx ast::LetStmt) {
        self.add_local(&let_stmt.ident, &let_stmt.ty, LocalKind::Ptr);
    }
}
