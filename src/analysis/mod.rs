use std::collections::HashMap;

use crate::ast;
use crate::ty::Ty;

#[derive(Debug)]
pub struct Ctxt<'ctx> {
    ty_mapping: HashMap<&'ctx str, Ty>,
}

impl<'ctx> Ctxt<'ctx> {
    pub fn new() -> Self {
        Ctxt {
            ty_mapping: HashMap::new(),
        }
    }

    pub fn set_type(&mut self, name: &'ctx str, ty: Ty) {
        let t = self.ty_mapping.insert(name, ty);
        if t.is_some() {
            panic!("ICE: dulplicated identifier? {name}");
        }
    }

    pub fn lookup_type(&self, name: &str) -> Option<&Ty> {
        self.ty_mapping.get(name)
    }
}

pub fn resolve(ctx: &Ctxt, krate: &ast::Crate) {}

pub struct Resolver;

impl ast::visitor::Visitor for Resolver {
    fn visit_crate(&mut self, krate: &ast::Crate) {
        todo!()
    }

    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        todo!()
    }

    fn visit_expr(&mut self, expr: &ast::Expr) {
        todo!()
    }

    fn visit_ident(&mut self, ident: &ast::Ident) {
        todo!()
    }
}
