use std::collections::HashMap;

use crate::ast::{self, Crate, Ident, NodeId, StmtKind};
use crate::middle::Ctxt;

pub fn resolve(ctx: &mut Ctxt, krate: &Crate) {
    let mut resolver = Resolver::new(ctx);
    ast::visitor::go(&mut resolver, krate);
}

struct Resolver<'ctx> {
    ctx: &'ctx mut Ctxt,
    // stack of (rib, )
    name_ribs: Vec<Rib>,
    next_rib_id: u32,
}

impl<'ctx> Resolver<'ctx> {
    fn new(ctx: &'ctx mut Ctxt) -> Self {
        Resolver {
            ctx,
            name_ribs: vec![],
            next_rib_id: 0,
        }
    }

    fn get_next_id(&mut self) -> u32 {
        let id = self.next_rib_id;
        self.next_rib_id += 1;
        id
    }

    fn new_rib(&mut self) -> Rib {
        Rib::new(self.get_next_id())
    }
}

impl<'ctx> ast::visitor::Visitor<'ctx> for Resolver<'ctx> {
    fn visit_func(&mut self, func: &'ctx ast::Func) {
        let mut r = self.new_rib();
        for (param, _) in &func.params {
            r.insert_binding(param.clone())
        }
        self.name_ribs.push(r);
    }

    fn visit_func_post(&mut self, func: &'ctx ast::Func) {
        let r = self.name_ribs.pop().unwrap();
        self.ctx.insert_rib(func.id, r);
    }

    fn visit_block(&mut self, _block: &'ctx ast::Block) {
        let r = self.new_rib();
        self.name_ribs.push(r);
    }

    fn visit_block_post(&mut self, block: &'ctx ast::Block) {
        let r = self.name_ribs.pop().unwrap();
        self.ctx.insert_rib(block.id, r)
    }

    fn visit_stmt(&mut self, stmt: &'ctx ast::Stmt) {
        if let StmtKind::Let(let_stmt) = &stmt.kind {
            // TODO: node id of statement
            self.name_ribs
                .last_mut()
                .unwrap()
                .insert_binding(let_stmt.ident.clone())
        }
    }
}

/// Struct representing a scope
/// ref: https://doc.rust-lang.org/stable/nightly-rustc/rustc_resolve/late/struct.Rib.html
#[derive(Debug)]
pub struct Rib {
    id: u32,
    bindings: HashMap<String, NodeId>,
}

impl Rib {
    fn new(rib_id: u32) -> Self {
        Rib {
            id: rib_id,
            bindings: HashMap::new(),
        }
    }

    pub fn insert_binding(&mut self, ident: Ident) {
        // FIXME: duplicate symbol?
        let _ = self.bindings.insert(ident.symbol, ident.id);
    }
}
