use crate::ast::{self, Ident, NodeId, StmtKind};
use std::{collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct Resolver {
    /// BlockOrFunc to rib mappings, which is set by resovler
    ribs: HashMap<NodeId, Rib>,
    // ident node to ribs mappings
    ident_to_ribs: HashMap<NodeId, Vec<Rib>>,

    name_ribs: Vec<Rib>,
    next_rib_id: u32,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            ribs: HashMap::new(),
            ident_to_ribs: HashMap::new(),
            name_ribs: vec![],
            next_rib_id: 0,
        }
    }

    pub fn insert_rib(&mut self, block_or_func_node_id: NodeId, rib: Rib) {
        self.ribs.insert(block_or_func_node_id, rib);
    }

    /// Resolve ident
    pub fn resolve_ident(&self, ident: &Ident) -> Option<NameBinding> {
        let ribs = self.ident_to_ribs.get(&ident.id).unwrap();
        Resolver::resolve_ident_from_ribs(ident, ribs)
    }

    // just utility function of resolve_ident
    fn resolve_ident_from_ribs(ident: &Ident, ribs: &[Rib]) -> Option<NameBinding> {
        for r in ribs.iter().rev() {
            let binding_kind = match &r.kind {
                RibKind::Block => BindingKind::Let,
                RibKind::Func => BindingKind::Arg,
            };
            if let Some(defined_ident_node_id) = r.bindings.get(&ident.symbol) {
                return Some(NameBinding::new(*defined_ident_node_id, binding_kind));
            }
        }
        None
    }

    fn get_next_id(&mut self) -> u32 {
        let id = self.next_rib_id;
        self.next_rib_id += 1;
        id
    }

    fn new_rib(&mut self, kind: RibKind) -> Rib {
        Rib::new(self.get_next_id(), kind)
    }

    fn set_ribs_to_ident_node(&mut self, ident_node_id: NodeId) {
        // FIXME: this `clone()` might be very slow.
        //   register all identifiers to name_ribs and after doing so, make it
        //   shared immutable using `Rc`
        self.ident_to_ribs
            .insert(ident_node_id, self.name_ribs.clone());
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct NameBinding {
    pub kind: BindingKind,
    defined_ident_node_id: NodeId,
}

impl NameBinding {
    fn new(defined_ident_node_id: NodeId, kind: BindingKind) -> Self {
        NameBinding {
            kind,
            defined_ident_node_id,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum BindingKind {
    Arg,
    Let,
}

impl<'ctx> ast::visitor::Visitor<'ctx> for Resolver {
    fn visit_func(&mut self, func: &'ctx ast::Func) {
        let mut r = self.new_rib(RibKind::Func);
        for (param, _) in &func.params {
            r.insert_binding(param.clone())
        }
        self.name_ribs.push(r);
    }

    fn visit_func_post(&mut self, func: &'ctx ast::Func) {
        let r = self.name_ribs.pop().unwrap();
        self.insert_rib(func.id, r);
    }

    fn visit_block(&mut self, _block: &'ctx ast::Block) {
        let r = self.new_rib(RibKind::Block);
        self.name_ribs.push(r);
    }

    fn visit_block_post(&mut self, block: &'ctx ast::Block) {
        let r = self.name_ribs.pop().unwrap();
        self.insert_rib(block.id, r)
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

    fn visit_ident(&mut self, ident: &'ctx ast::Ident) {
        self.set_ribs_to_ident_node(ident.id);
    }
}

/// Struct representing a scope
/// ref: https://doc.rust-lang.org/stable/nightly-rustc/rustc_resolve/late/struct.Rib.html
#[derive(Debug, Clone)]
pub struct Rib {
    id: u32,
    kind: RibKind,
    bindings: HashMap<Rc<String>, NodeId>,
}

#[derive(Debug, Clone)]
enum RibKind {
    Func,
    Block,
}

impl Rib {
    fn new(rib_id: u32, kind: RibKind) -> Self {
        Rib {
            id: rib_id,
            kind,
            bindings: HashMap::new(),
        }
    }

    // TODO: shadowing
    pub fn insert_binding(&mut self, ident: Ident) {
        // FIXME: duplicate symbol?
        let _ = self.bindings.insert(ident.symbol, ident.id);
    }
}
