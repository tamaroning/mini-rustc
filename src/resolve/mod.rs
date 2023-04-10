mod resolve_crate;
mod resolve_toplevel;

use self::resolve_toplevel::ResolveTopLevel;
use crate::{ast::NodeId, span::Ident};
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

/// Struct representing a scope
/// ref: https://doc.rust-lang.org/stable/nightly-rustc/rustc_resolve/late/struct.Rib.html
#[derive(Debug)]
pub struct Rib {
    id: RibId,
    // let a = 0; a
    //            ^
    // ↓
    // let a = 0; a
    //     ^
    bindings: HashMap<Rc<String>, Binding>,
    parent: Option<RibId>,
}

type RibId = u32;

impl Rib {
    fn new(rib_id: u32, parent: Option<RibId>) -> Self {
        Rib {
            id: rib_id,
            bindings: HashMap::new(),
            parent,
        }
    }

    // TODO: shadowing
    pub fn insert_binding(&mut self, symbol: Rc<String>, binding: Binding) {
        // FIXME: duplicate symbol?
        self.bindings.insert(symbol, binding);
    }
}

#[derive(Debug)]
pub struct Resolver {
    resolve_toplevel: ResolveTopLevel,

    /// Crate/Func/Block to rib mappings, which is set by resovler
    ribs: HashMap<NodeId, RibId>,
    // local and param node to ribs mappings
    ident_to_rib: HashMap<Ident, RibId>,
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
            ident_to_rib: HashMap::new(),
            current_ribs: vec![],
            current_cpath: CanonicalPath::empty(),
            interned: HashMap::new(),
            next_rib_id: 0,
        }
    }

    pub fn resolve_ident(&mut self, ident: &Ident) -> Option<Rc<Binding>> {
        if let Some(b) = self.resolve_toplevel.search_item(&ident.symbol) {
            Some(b)
        } else if let Some(rib) = self.ident_to_rib.get(&ident) {
            let binding = Rc::new(self.resolve_segment_from_rib(&ident.symbol, *rib)?);
            Some(binding)
        } else {
            None
        }
    }

    /// Utility function of resolve_ident

    // just search in ancester ribs for now
    // TODO: search in ribs other than ancester
    fn resolve_segment_from_rib(&self, seg: &Rc<String>, rib_id: RibId) -> Option<Binding> {
        let mut current_rib = self.get_rib(rib_id);

        loop {
            if let Some(binding) = current_rib.bindings.get(seg) {
                return Some(binding.clone());
            } else if let Some(parent_rib_id) = current_rib.parent {
                current_rib = self.get_rib(parent_rib_id);
            } else {
                break;
            }
        }
        None
    }

    fn get_rib(&self, rib_id: RibId) -> &Rib {
        self.interned.get(&rib_id).unwrap()
    }
}
