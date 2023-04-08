use crate::ast::{self, Ident, NodeId, StmtKind};
use std::{collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct Resolver {
    /// Crate/Func/Block to rib mappings, which is set by resovler
    ribs: HashMap<NodeId, RibId>,
    // all ident node to ribs mappings
    ident_to_ribs: HashMap<NodeId, Vec<RibId>>,
    // current name ribs
    name_ribs: Vec<RibId>,
    // interned ribs
    interned: HashMap<RibId, Rib>,
    next_rib_id: u32,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            ribs: HashMap::new(),
            ident_to_ribs: HashMap::new(),
            name_ribs: vec![],
            interned: HashMap::new(),
            next_rib_id: 0,
        }
    }

    fn get_current_rib_mut(&mut self) -> &mut Rib {
        let current_rib_id = self.name_ribs.last().unwrap();
        self.interned.get_mut(&current_rib_id).unwrap()
    }

    fn get_rib(&self, rib_id: RibId) -> &Rib {
        self.interned.get(&rib_id).unwrap()
    }

    fn push_rib(&mut self, rib_kind: RibKind, node_id: NodeId) {
        let rib = Rib::new(self.get_next_id(), rib_kind);
        self.name_ribs.push(rib.id);
        self.ribs.insert(node_id, rib.id);
        self.interned.insert(rib.id, rib);
    }

    fn pop_rib(&mut self) {
        self.name_ribs.pop().unwrap();
    }

    /// Resolve identifier (local variable, parameters, function name)
    pub fn resolve_ident(&self, ident: &Ident) -> Option<NameBinding> {
        let ribs = self.ident_to_ribs.get(&ident.id).unwrap();
        self.resolve_ident_from_ribs(ident, ribs)
    }

    // just utility function of resolve_ident
    fn resolve_ident_from_ribs(&self, ident: &Ident, ribs: &[RibId]) -> Option<NameBinding> {
        for rib_id in ribs.iter().rev() {
            let rib = self.get_rib(*rib_id);
            let binding_kind = match &rib.kind {
                RibKind::Block => BindingKind::Let,
                RibKind::Func => BindingKind::Arg,
                RibKind::Crate => BindingKind::Item,
            };
            if let Some(defined_ident_node_id) = rib.bindings.get(&ident.symbol) {
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
    Item,
}

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

/// Struct representing a scope
/// ref: https://doc.rust-lang.org/stable/nightly-rustc/rustc_resolve/late/struct.Rib.html
#[derive(Debug, Clone)]
pub struct Rib {
    id: u32,
    kind: RibKind,
    // let a = 0; a
    //            ^
    // ↓
    // let a = 0; a
    //     ^
    bindings: HashMap<Rc<String>, NodeId>,
}

type RibId = u32;

#[derive(Debug, Clone)]
enum RibKind {
    Func,
    Block,
    Crate,
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
    pub fn insert_binding(&mut self, ident: &Ident) {
        // FIXME: duplicate symbol?
        self.bindings.insert(ident.symbol.clone(), ident.id);
    }
}
