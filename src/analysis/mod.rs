use std::collections::HashMap;

use crate::ast::visitor::{go, Visitor};
use crate::ast::{self, LetStmt};
use crate::ty::Ty;

#[derive(Debug)]
pub struct Ctxt<'ctx> {
    ty_mapping: HashMap<&'ctx String, Ty>,
    pub dump_enabled: bool,
}

impl<'ctx> Ctxt<'ctx> {
    pub fn new(dump_enabled: bool) -> Self {
        Ctxt {
            ty_mapping: HashMap::new(),
            dump_enabled,
        }
    }

    pub fn set_type(&mut self, name: &'ctx String, ty: Ty) {
        let t = self.ty_mapping.insert(name, ty);
        if t.is_some() {
            panic!("ICE: dulplicated identifier? {name}");
        }
    }

    pub fn lookup_type(&self, name: &String) -> Option<&Ty> {
        self.ty_mapping.get(name)
    }

    pub fn get_all_local_vars(&self) -> Vec<(&String, &Ty)> {
        let v: Vec<(&String, &Ty)> = self.ty_mapping.iter().map(|(m, ty)| (*m, ty)).collect();
        v
    }
}

pub fn resolve<'ctx, 'a>(ctx: &'a mut Ctxt<'ctx>, krate: &'ctx ast::Crate) {
    let resolver: &mut dyn Visitor = &mut Resolver { ctx };
    go(resolver, krate);
}

pub struct Resolver<'ctx, 'a> {
    ctx: &'a mut Ctxt<'ctx>,
}

impl<'ctx> ast::visitor::Visitor<'ctx> for Resolver<'ctx, '_> {
    fn visit_crate(&mut self, _krate: &'ctx ast::Crate) {}

    fn visit_stmt(&mut self, _stmt: &'ctx ast::Stmt) {}

    fn visit_expr(&mut self, _expr: &'ctx ast::Expr) {}

    fn visit_let_stmt(&mut self, let_stmt: &'ctx ast::LetStmt) {
        let LetStmt { ident } = &let_stmt;
        self.ctx.set_type(&ident.symbol, Ty::I32);
    }

    fn visit_ident(&mut self, _ident: &'ctx ast::Ident) {}
}
