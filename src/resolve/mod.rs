mod resolve_crate;

use crate::ast::Ident;
use crate::ast::NodeId;
use std::{collections::HashMap, rc::Rc};

#[derive(Clone, Eq)]
pub struct CanonicalPath {
    pub res: Res,
    // `mod_a::func_f::var_a` => ["mod_a", "func_f", "var_a"]
    pub segments: Vec<Rc<String>>,
}

impl CanonicalPath {
    pub fn empty() -> Self {
        CanonicalPath {
            res: Res::Error,
            segments: vec![],
        }
    }
    pub fn push_segment(&mut self, seg: Rc<String>, res: Res) {
        self.segments.push(seg);
        self.res = res;
    }

    pub fn pop_segment(&mut self) -> Option<Rc<String>> {
        self.segments.pop()
    }
}

impl std::cmp::PartialEq for CanonicalPath {
    fn eq(&self, other: &Self) -> bool {
        self.res == other.res
    }

    fn ne(&self, other: &Self) -> bool {
        self.res != other.res
    }
}

impl std::fmt::Display for CanonicalPath {
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

impl std::fmt::Debug for CanonicalPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}", self, self.res)
    }
}

impl std::hash::Hash for CanonicalPath {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.res.hash(state);
    }
}

#[derive(Debug)]
pub struct Resolver {
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
    cpath_cache: HashMap<NodeId, Rc<CanonicalPath>>,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            ribs: HashMap::new(),
            ident_to_ribs: HashMap::new(),
            current_ribs: vec![],
            current_cpath: CanonicalPath::empty(),
            interned: HashMap::new(),
            next_rib_id: 0,
            cpath_cache: HashMap::new(),
        }
    }

    pub fn resolve_ident(&mut self, ident: &Ident) -> Option<Rc<CanonicalPath>> {
        if let Some(cpath) = self.cpath_cache.get(&ident.id) {
            Some(Rc::clone(cpath))
        } else {
            let ribs = self.ident_to_ribs.get(&ident.id).unwrap();
            let cpath = Rc::new(self.resolve_segment_from_ribs(&ident.symbol, ribs)?);
            self.cpath_cache.insert(ident.id, Rc::clone(&cpath));
            Some(cpath)
        }
    }

    // just utility function of resolve_ident
    fn resolve_segment_from_ribs(&self, seg: &Rc<String>, ribs: &[RibId]) -> Option<CanonicalPath> {
        for rib_id in ribs.iter().rev() {
            let rib = self.get_rib(*rib_id);
            if let Some(res) = rib.bindings.get(seg) {
                let mut cpath = rib.cpath.clone();
                cpath.push_segment(Rc::clone(seg), *res);
                return Some(cpath);
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
    bindings: HashMap<Rc<String>, Res>,
}

type RibId = u32;

// link to NodeId of ast::Ident
// https://doc.rust-lang.org/stable/nightly-rustc/rustc_hir/def/enum.Res.html
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Res {
    Crate(NodeId),
    Func(NodeId),
    Let(NodeId),
    Param(NodeId),
    Error,
}

impl Res {
    pub fn is_param(&self) -> bool {
        matches!(self, Res::Param(_))
    }

    pub fn is_let(&self) -> bool {
        matches!(self, Res::Let(_))
    }
}

impl Rib {
    fn new(rib_id: u32, cpath: CanonicalPath) -> Self {
        Rib {
            id: rib_id,
            cpath,
            bindings: HashMap::new(),
        }
    }

    // TODO: shadowing
    pub fn insert_binding(&mut self, symbol: Rc<String>, res: Res) {
        // FIXME: duplicate symbol?
        self.bindings.insert(symbol, res);
    }
}
