pub mod ty;

use crate::ast::{self, Crate, NodeId, Path};
//use crate::hir::{self, HirId, LocalDefId};
//use crate::hir::HirId;
use crate::middle::ty::{AdtDef, Ty};
use crate::resolve::{Binding, CanonicalPath, Resolver};
use crate::span::Ident;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Ctxt<'ctx> {
    pub dump_enabled: bool,
    // Set during name resolution stage
    resolver: Resolver,

    /// HIR root module
    //hir_root_module: LocalDefId,
    //hir_items: HashMap<LocalDefId, hir::Item<'ctx>>,
    //hir_ty_mappings: HashMap<HirId, Rc<Ty>>,
    phantom: std::marker::PhantomData<&'ctx ()>,

    // Set during typecheck stage
    /// Expr/Stmt/Block to type mappings
    ty_mappings: HashMap<NodeId, Rc<Ty>>,
    /// local variables, paramters, function-name to type mappings
    pub name_ty_mappings: HashMap<Rc<Binding>, Rc<Ty>>,
    // TODO: use NameBinding
    adt_defs: HashMap<Rc<CanonicalPath>, Rc<AdtDef>>,
    // Set during rvalue anlaysis stage
    // all node ids of place expressions
    // ref: https://doc.rust-lang.org/reference/expressions.html?highlight=rvalue#place-expressions-and-value-expressions
    // lvalues: HashSet<NodeId>,
}

impl<'ctx> Ctxt<'ctx> {
    pub fn new(dump_enabled: bool) -> Self {
        Ctxt {
            dump_enabled,
            resolver: Resolver::new(),

            //hir_root_module: LocalDefId::dummy(),
            //hir_items: HashMap::new(),
            //hir_ty_mappings: HashMap::new(),
            phantom: std::marker::PhantomData::default(),

            ty_mappings: HashMap::new(),
            name_ty_mappings: HashMap::new(),
            adt_defs: HashMap::new(),
            // lvalues: HashSet::new(),
        }
    }

    // Resolution Stage

    pub fn run_resolver(&mut self, krate: &Crate) {
        ast::visitor::go(&mut self.resolver, krate);
    }

    /// Resolve identifiers in var decls (func params or local variables) to canonical paths
    pub fn get_binding(&mut self, ident: &Ident) -> Option<Rc<Binding>> {
        self.resolver.get_binding(ident)
    }

    /// Resolve paths to canonical paths
    pub fn resolve_path(&mut self, path: &Path) -> Option<Rc<Binding>> {
        self.resolver.resolve_path(path)
    }

    pub fn dump_ribs(&self) {
        self.resolver.dump_ribs();
    }

    pub fn dump_resolution(&self) {
        self.resolver.dump_resolution();
    }

    // Typecheck Stage

    pub fn insert_type(&mut self, node_id: NodeId, ty: Rc<Ty>) {
        self.ty_mappings.insert(node_id, ty);
    }

    pub fn get_type(&self, node_id: NodeId) -> Rc<Ty> {
        Rc::clone(self.ty_mappings.get(&node_id).unwrap())
    }

    pub fn lookup_name_type(&self, binding: &Binding) -> Option<Rc<Ty>> {
        self.name_ty_mappings.get(binding).map(Rc::clone)
    }

    pub fn set_name_type(&mut self, binding: Rc<Binding>, fn_ty: Rc<Ty>) {
        self.name_ty_mappings.insert(binding, fn_ty);
    }

    pub fn lookup_adt_def(&self, cpath: &CanonicalPath) -> Option<Rc<AdtDef>> {
        self.adt_defs.get(cpath).map(Rc::clone)
    }

    pub fn set_adt_def(&mut self, cpath: Rc<CanonicalPath>, adt: AdtDef) {
        self.adt_defs.insert(cpath, Rc::new(adt));
    }

    pub fn get_adt_defs(&self) -> &HashMap<Rc<CanonicalPath>, Rc<AdtDef>> {
        &self.adt_defs
    }

    // Rvalue analysis stage
    /*
    pub fn register_lvalue(&mut self, node_id: NodeId) {
        self.lvalues.insert(node_id);
    }

    pub fn is_lvalue(&mut self, node_id: NodeId) -> bool {
        self.lvalues.contains(&node_id)
    }
    */

    // Codegen stage
}
