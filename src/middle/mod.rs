pub mod ty;

use crate::ast::{self, Crate, NodeId};
use crate::middle::ty::{AdtDef, Ty};
use crate::resolve::Resolver;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Debug)]
pub struct Ctxt {
    pub dump_enabled: bool,
    // Set during name resolution stage
    pub resolver: Resolver,

    // Set during typecheck stage
    /// ExprOrStmtOrBlock to type mappings
    ty_mappings: HashMap<NodeId, Rc<Ty>>,
    fn_types: HashMap<Rc<String>, Rc<Ty>>,
    adt_defs: HashMap<Rc<String>, Rc<AdtDef>>,

    // Set during rvalue anlaysis stage
    /// all node ids of place expressions
    /// ref: https://doc.rust-lang.org/reference/expressions.html?highlight=rvalue#place-expressions-and-value-expressions
    lvalues: HashSet<NodeId>,
}

impl<'ctx> Ctxt {
    pub fn new(dump_enabled: bool) -> Self {
        Ctxt {
            dump_enabled,
            resolver: Resolver::new(),
            ty_mappings: HashMap::new(),
            fn_types: HashMap::new(),
            adt_defs: HashMap::new(),
            lvalues: HashSet::new(),
        }
    }

    // Resolution Stage

    pub fn resolve(&mut self, krate: &'ctx Crate) {
        ast::visitor::go(&mut self.resolver, krate);
    }

    // Typecheck Stage

    pub fn insert_type(&mut self, node_id: NodeId, ty: Rc<Ty>) {
        self.ty_mappings.insert(node_id, ty);
    }

    pub fn get_type(&self, node_id: NodeId) -> Rc<Ty> {
        Rc::clone(self.ty_mappings.get(&node_id).unwrap())
    }

    pub fn lookup_fn_type(&self, func_name: &String) -> Option<Rc<Ty>> {
        self.fn_types.get(func_name).map(Rc::clone)
    }

    pub fn set_fn_type(&mut self, func_name: Rc<String>, fn_ty: Rc<Ty>) {
        self.fn_types.insert(func_name, fn_ty);
    }

    pub fn lookup_adt_def(&self, adt_name: &String) -> Option<Rc<AdtDef>> {
        self.adt_defs.get(adt_name).map(Rc::clone)
    }

    pub fn set_adt_def(&mut self, name: Rc<String>, adt: AdtDef) {
        self.adt_defs.insert(name, Rc::new(adt));
    }

    pub fn get_adt_defs(&self) -> &HashMap<Rc<String>, Rc<AdtDef>> {
        &self.adt_defs
    }

    // Rvalue analysis stage
    pub fn register_lvalue(&mut self, node_id: NodeId) {
        self.lvalues.insert(node_id);
    }

    pub fn is_lvalue(&mut self, node_id: NodeId) -> bool {
        self.lvalues.contains(&node_id)
    }

    // Codegen stage
}
