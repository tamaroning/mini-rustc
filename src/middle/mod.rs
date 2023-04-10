pub mod ty;

use crate::ast::{self, Crate, NodeId};
use crate::middle::ty::{AdtDef, Ty};
use crate::resolve::{Binding, CanonicalPath, Resolver};
use crate::span::Ident;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Debug)]
pub struct Ctxt {
    pub dump_enabled: bool,
    // Set during name resolution stage
    resolver: Resolver,

    // Set during typecheck stage
    /// Expr/Stmt/Block to type mappings
    ty_mappings: HashMap<NodeId, Rc<Ty>>,
    /// local variables, paramters, function-name to type mappings
    pub name_ty_mappings: HashMap<Rc<Binding>, Rc<Ty>>,
    // TODO: use NameBinding
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
            name_ty_mappings: HashMap::new(),
            adt_defs: HashMap::new(),
            lvalues: HashSet::new(),
        }
    }

    // Resolution Stage

    pub fn resolve(&mut self, krate: &'ctx Crate) {
        ast::visitor::go(&mut self.resolver, krate);
    }

    /// Resolve identifier (local variable, parameters, function name)
    pub fn resolve_ident(&mut self, ident: &Ident) -> Option<Rc<Binding>> {
        self.resolver.resolve_ident(ident)
    }

    // Typecheck Stage

    pub fn insert_type(&mut self, node_id: NodeId, ty: Rc<Ty>) {
        self.ty_mappings.insert(node_id, ty);
    }

    pub fn get_type(&self, node_id: NodeId) -> Rc<Ty> {
        Rc::clone(self.ty_mappings.get(&node_id).unwrap())
    }

    pub fn lookup_cpath_type(&self, name: &Rc<Binding>) -> Option<Rc<Ty>> {
        self.name_ty_mappings.get(name).map(Rc::clone)
    }

    pub fn set_cpath_type(&mut self, name: Rc<Binding>, fn_ty: Rc<Ty>) {
        self.name_ty_mappings.insert(name, fn_ty);
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
