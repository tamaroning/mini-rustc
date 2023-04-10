mod resolve_crate;
mod resolve_toplevel;

use self::resolve_toplevel::ResolveTopLevel;
use crate::ast::Ident;
use crate::ast::NodeId;
use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Binding {
    pub cpath: CanonicalPath,
    pub kind: BindingKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BindingKind {
    Crate,
    Mod,
    Func,
    Struct,
    Let,
    Param,
}

impl BindingKind {
    pub fn is_param(&self) -> bool {
        matches!(self, BindingKind::Param)
    }

    pub fn is_let(&self) -> bool {
        matches!(self, BindingKind::Let)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CanonicalPath {
    segments: Vec<Rc<String>>,
}

impl CanonicalPath {
    fn empty() -> Self {
        CanonicalPath { segments: vec![] }
    }

    fn push_seg(&mut self, seg: Rc<String>) {
        self.segments.push(seg);
    }

    fn pop_seg(&mut self) -> Option<Rc<String>> {
        self.segments.pop()
    }
}

impl std::fmt::Debug for CanonicalPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, seg) in self.segments.iter().enumerate() {
            if i != 0 {
                write!(f, "::")?;
            }
            write!(f, "{}", *seg)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Resolver {
    resolve_toplevel: ResolveTopLevel,

    /// Crate/Func/Block to rib mappings, which is set by resovler
    ribs: HashMap<NodeId, RibId>,
    // all ident node to ribs mappings
    ident_to_ribs: HashMap<NodeId, Vec<RibId>>,
    // stack of urrent name ribs
    current_ribs: Vec<RibId>,
    // current canonical path
    current_cpath: CanonicalPath,
    next_rib_id: u32,
    // interned ribs
    interned: HashMap<RibId, Rib>,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            resolve_toplevel: ResolveTopLevel::new(),
            ribs: HashMap::new(),
            ident_to_ribs: HashMap::new(),
            current_ribs: vec![],
            current_cpath: CanonicalPath::empty(),
            interned: HashMap::new(),
            next_rib_id: 0,
        }
    }

    pub fn resolve_ident(&mut self, ident: &Ident) -> Option<Rc<Binding>> {
        if let Some(b) = self.resolve_toplevel.search_ident(&ident.symbol) {
            Some(b)
        } else if let Some(ribs) = self.ident_to_ribs.get(&ident.id) {
            let binding = Rc::new(self.resolve_segment_from_ribs(&ident.symbol, ribs)?);
            Some(binding)
        } else {
            None
        }
    }

    // just utility function of resolve_ident
    fn resolve_segment_from_ribs(&self, seg: &Rc<String>, ribs: &[RibId]) -> Option<Binding> {
        for rib_id in ribs.iter().rev() {
            let rib = self.get_rib(*rib_id);
            if let Some(binding) = rib.bindings.get(seg) {
                return Some(binding.clone());
            }
        }
        None
    }

    fn get_rib(&self, rib_id: RibId) -> &Rib {
        self.interned.get(&rib_id).unwrap()
    }
}

/// Struct representing a scope
/// ref: https://doc.rust-lang.org/stable/nightly-rustc/rustc_resolve/late/struct.Rib.html
#[derive(Debug)]
pub struct Rib {
    id: RibId,
    // optional name
    cpath: CanonicalPath,
    // let a = 0; a
    //            ^
    // ↓
    // let a = 0; a
    //     ^
    bindings: HashMap<Rc<String>, Binding>,
}

type RibId = u32;

impl Rib {
    fn new(rib_id: u32, cpath: CanonicalPath) -> Self {
        Rib {
            id: rib_id,
            cpath,
            bindings: HashMap::new(),
        }
    }

    // TODO: shadowing
    pub fn insert_binding(&mut self, symbol: Rc<String>, binding: Binding) {
        // FIXME: duplicate symbol?
        self.bindings.insert(symbol, binding);
    }
}
