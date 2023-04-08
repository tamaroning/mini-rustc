use super::{Resolver, RibKind};
use crate::ast::{self, StmtKind};

impl<'ctx> ast::visitor::Visitor<'ctx> for Resolver {
    fn visit_crate(&mut self, krate: &'ctx ast::Crate) {
        // push new rib
        self.push_rib(RibKind::Crate, krate.id);
    }

    fn visit_crate_post(&mut self, _krate: &'ctx ast::Crate) {
        self.pop_rib();
    }

    fn visit_func(&mut self, func: &'ctx ast::Func) {
        // insert func name
        self.get_current_rib_mut().insert_binding(&func.name);

        // push new rib
        self.push_rib(RibKind::Func, func.id);

        // insert parameters
        for (param, _) in &func.params {
            self.get_current_rib_mut().insert_binding(&param)
        }
    }

    fn visit_func_post(&mut self, _: &'ctx ast::Func) {
        // pop current rib
        self.pop_rib();
    }

    fn visit_block(&mut self, block: &'ctx ast::Block) {
        // push new rib
        self.push_rib(RibKind::Block, block.id);
    }

    fn visit_block_post(&mut self, _: &'ctx ast::Block) {
        // pop current rib
        self.pop_rib();
    }

    fn visit_stmt(&mut self, stmt: &'ctx ast::Stmt) {
        if let StmtKind::Let(let_stmt) = &stmt.kind {
            // insert local variables
            self.get_current_rib_mut().insert_binding(&let_stmt.ident)
        }
    }

    fn visit_ident(&mut self, ident: &'ctx ast::Ident) {
        self.set_ribs_to_ident_node(ident.id);
    }
}
