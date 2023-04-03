use crate::ast::{self, Crate, Ident, NodeId, StmtKind};
use crate::middle::Ctxt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Resolver {
    /// BlockOrFunc to rib mappings, which is set by resovler
    ribs: HashMap<NodeId, Rc<Rib>>,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            ribs: HashMap::new(),
        }
    }

    pub fn insert_rib(&mut self, node_id: NodeId, rib: Rc<Rib>) {
        self.ribs.insert(node_id, rib);
    }

    pub fn get_rib(&self, node_id: NodeId) -> Rc<Rib> {
        Rc::clone(self.ribs.get(&node_id).unwrap())
    }

    // TODO: self is not needed
    pub fn resolve_ident(&self, ident: &Ident, ribs: &Vec<Rc<Rib>>) -> Option<NameBinding> {
        for r in ribs.iter().rev() {
            if let Some(defined_ident_node_id) = r.bindings.get(&ident.symbol) {
                return Some(NameBinding::new(*defined_ident_node_id));
            }
        }
        None
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct NameBinding {
    defined_ident_node_id: NodeId,
}

impl NameBinding {
    fn new(defined_ident_node_id: NodeId) -> Self {
        NameBinding {
            defined_ident_node_id,
        }
    }
}

pub fn analyze(ctx: &mut Ctxt, krate: &Crate) {
    let mut analyzer = RibAnlyzer::new(ctx);
    ast::visitor::go(&mut analyzer, krate);
}

struct RibAnlyzer<'ctx> {
    pub ctx: &'ctx mut Ctxt,
    name_ribs: Vec<Rib>,
    next_rib_id: u32,
}

impl<'ctx> RibAnlyzer<'ctx> {
    fn new(ctx: &'ctx mut Ctxt) -> Self {
        RibAnlyzer {
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

impl<'ctx> ast::visitor::Visitor<'ctx> for RibAnlyzer<'ctx> {
    fn visit_func(&mut self, func: &'ctx ast::Func) {
        let mut r = self.new_rib();
        for (param, _) in &func.params {
            r.insert_binding(param.clone())
        }
        self.name_ribs.push(r);
    }

    fn visit_func_post(&mut self, func: &'ctx ast::Func) {
        let r = self.name_ribs.pop().unwrap();
        self.ctx.resolver.insert_rib(func.id, Rc::new(r));
    }

    fn visit_block(&mut self, _block: &'ctx ast::Block) {
        let r = self.new_rib();
        self.name_ribs.push(r);
    }

    fn visit_block_post(&mut self, block: &'ctx ast::Block) {
        let r = self.name_ribs.pop().unwrap();
        self.ctx.resolver.insert_rib(block.id, Rc::new(r))
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

    // TODO: shadowing
    pub fn insert_binding(&mut self, ident: Ident) {
        // FIXME: duplicate symbol?
        let _ = self.bindings.insert(ident.symbol, ident.id);
    }
}
