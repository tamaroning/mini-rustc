use std::{collections::HashMap, rc::Rc};

use crate::{ast, middle::ty::Ty, resolve::NameBinding};

use super::{Codegen, LLReg, LLTy};

#[derive(Debug)]
pub struct Frame {
    locals: HashMap<NameBinding, Rc<LLReg>>,
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            locals: HashMap::new(),
        }
    }

    pub fn get_local(&self, name: &NameBinding) -> Rc<LLReg> {
        Rc::clone(self.locals.get(name).unwrap())
    }

    pub fn get_locals(&self) -> &HashMap<NameBinding, Rc<LLReg>> {
        &self.locals
    }
}

pub struct VisitFrame<'a, 'b, 'c> {
    pub codegen: &'a mut Codegen<'b>,
    pub frame: &'c mut Frame,
}

impl VisitFrame<'_, '_, '_> {
    fn add_local(&mut self, ident: &ast::Ident, ty: &Rc<Ty>) {
        // TODO: align
        // let align = self.codegen.ctx.get_align(ty);
        let reg_ty = LLTy::Ptr(Box::new(self.codegen.ty_to_llty(ty)));
        let name_binding = self.codegen.ctx.resolver.resolve_ident(ident).unwrap();
        let reg_name = format!("%{}", ident.symbol);
        let reg = LLReg::new(reg_name, reg_ty);
        self.frame.locals.insert(name_binding, Rc::new(reg));
    }
}

impl<'ctx: 'a, 'a> ast::visitor::Visitor<'ctx> for VisitFrame<'_, '_, '_> {
    fn visit_let_stmt(&mut self, let_stmt: &'ctx ast::LetStmt) {
        self.add_local(&let_stmt.ident, &let_stmt.ty);
    }
}
