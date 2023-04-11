mod resolve_crate;

use crate::{ast::Path, span::Ident};
use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Binding {
    pub cpath: Rc<CanonicalPath>,
    pub kind: BindingKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BindingKind {
    Mod,
    Item,
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

    pub fn demangle(&self) -> String {
        let mut s = String::new();
        for (i, seg) in self.segments.iter().enumerate() {
            // skip first `crate`
            if i == 0 {
                continue;
            } else if i != 1 && i != self.segments.len() - 1 {
                s.push_str("..");
            }
            s.push_str(&seg);
        }
        s
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
    bindings: HashMap<Rc<String>, Rc<Binding>>,
    parent: Option<RibId>,
    cpath: CanonicalPath,
}

type RibId = u32;

impl Rib {
    fn new(rib_id: u32, parent: Option<RibId>, cpath: CanonicalPath) -> Self {
        Rib {
            id: rib_id,
            bindings: HashMap::new(),
            parent,
            cpath,
        }
    }

    // TODO: shadowing
    pub fn insert_binding(&mut self, symbol: Rc<String>, binding: Binding) {
        // FIXME: duplicate symbol?
        self.bindings.insert(symbol, Rc::new(binding));
    }
}

#[derive(Debug)]
pub struct Resolver {
    // Lookup map
    def_to_rib: HashMap<Ident, RibId>,
    // Lookup map
    use_to_rib: HashMap<Path, RibId>,
    // stack of urrent name ribs
    current_ribs: Vec<RibId>,
    // current canonical path
    current_cpath: CanonicalPath,
    next_rib_id: u32,
    // interned ribs
    interned: HashMap<RibId, Rib>,

    cache: HashMap<Path, Rc<Binding>>,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            def_to_rib: HashMap::new(),
            use_to_rib: HashMap::new(),
            current_ribs: vec![],
            current_cpath: CanonicalPath::empty(),
            interned: HashMap::new(),
            next_rib_id: 0,

            cache: HashMap::new(),
        }
    }

    pub fn dump_ribs(&self) {
        println!("===== Ribs in resolver =====");
        for (rib_id, rib) in &self.interned {
            println!("{} => [", rib_id);
            println!("\tcpath: {:?}", rib.cpath);
            for (s, binding) in &rib.bindings {
                println!("\t\"{}\" => {:?}, ", s, binding);
            }
            println!("\tparent: {:?}", rib.parent);
            println!("]");
        }
        for (ident, rib_id) in &self.def_to_rib {
            println!("def of {:?} =>  {}, ", ident, rib_id);
        }
        for (ident, rib_id) in &self.use_to_rib {
            println!("use of {:?} =>  {}, ", ident, rib_id);
        }
        println!();
        println!("============================");
    }

    pub fn dump_resolution(&self) {
        println!("===== Resolved names =======");
        for (ident, binding) in &self.cache {
            println!("{:?} => {:?}", ident, binding);
        }
        println!("============================");
    }

    /// Resolve identifiers in declaration nodes (func params or local variables) to canonical paths
    pub fn resolve_var_or_item_decl(&mut self, ident: &Ident) -> Option<Rc<Binding>> {
        if let Some(rib_id) = self.def_to_rib.get(ident) {
            let rib = self.get_rib(*rib_id);
            if let Some(binding) = rib.bindings.get(&ident.symbol) {
                return Some(binding.clone());
            } else {
                panic!(
                    "ICE: {:?} is in def_to_rib, but rib does not contain its def",
                    ident
                );
            }
        } else {
            None
        }
    }

    /// Resolve paths to canonical paths
    pub fn resolve_path(&mut self, path: &Path) -> Option<Rc<Binding>> {
        if let Some(binding) = self.cache.get(&path) {
            Some(Rc::clone(binding))
        } else if let Some(rib) = self.use_to_rib.get(path) {
            let binding = self.resolve_segment_from_rib(path, *rib)?;
            let binding = Rc::clone(&binding);
            self.cache.insert(path.clone(), Rc::clone(&binding));
            Some(binding)
        } else {
            None
        }
    }

    /// Utility function of resolve_ident

    // just search in ancester ribs for now
    // TODO: search in ribs other than ancester
    fn resolve_segment_from_rib(&self, path: &Path, rib_id: RibId) -> Option<Rc<Binding>> {
        let seg = &path.segments.last().unwrap().symbol;
        let mut current_rib = self.get_rib(rib_id);
        loop {
            if let Some(binding) = current_rib.bindings.get(seg) {
                return Some(Rc::clone(binding));
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
