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

    fn krate() -> Self {
        CanonicalPath {
            segments: vec![Rc::new("crate".to_string())],
        }
    }

    fn from_path(prefix: &CanonicalPath, path: &Path) -> Self {
        let mut ret = prefix.clone();
        for seg in &path.segments {
            ret.segments.push(Rc::clone(&seg.symbol));
        }
        ret
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
            }
            s.push_str(&seg);
            if i != self.segments.len() - 1 {
                s.push_str("..");
            }
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
    kind: RibKind,
    cpath: CanonicalPath,
    bindings: HashMap<Rc<String>, Rc<Binding>>,
    parent: Option<RibId>,
    children: Vec<RibId>,
}

type RibId = u32;
const DUMMY_RIB_ID: u32 = u32::MAX;

#[derive(Debug, PartialEq, Eq)]
pub enum RibKind {
    Mod,
    Func,
    Block,
}

impl Rib {
    fn new(rib_id: u32, kind: RibKind, parent: Option<RibId>, cpath: CanonicalPath) -> Self {
        Rib {
            id: rib_id,
            kind,
            bindings: HashMap::new(),
            parent,
            children: vec![],
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
    crate_rib_id: RibId,

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
            crate_rib_id: DUMMY_RIB_ID,

            cache: HashMap::new(),
        }
    }

    fn get_rib(&self, rib_id: RibId) -> &Rib {
        self.interned.get(&rib_id).unwrap()
    }

    fn get_rib_mut(&mut self, rib_id: RibId) -> &mut Rib {
        self.interned.get_mut(&rib_id).unwrap()
    }

    pub fn dump_ribs(&self) {
        println!("===== Ribs in resolver =====");
        for (rib_id, rib) in &self.interned {
            println!("{} => [", rib_id);
            println!("\tcpath: {:?}", rib.cpath);
            println!("\tkind: {:?}", rib.kind);
            for (s, binding) in &rib.bindings {
                println!("\t\"{}\" => {:?}, ", s, binding);
            }
            println!("\tparent: {:?}", rib.parent);
            println!("\tchildren: {:?}", rib.children);
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
            let binding = self.resolve_path_from_rib(path, *rib)?;
            let binding = Rc::clone(&binding);
            self.cache.insert(path.clone(), Rc::clone(&binding));
            Some(binding)
        } else {
            None
        }
    }

    /// Items visibie from a name space: `crate`, siblings items, and `use`d namespace
    /// `path`: path in question
    /// `rib_id`s: RibId of rib where path is used
    fn resolve_path_from_rib(&self, path: &Path, rib_id: RibId) -> Option<Rc<Binding>> {
        let emp_cpath = CanonicalPath::empty();
        let crate_cpath = CanonicalPath::krate();
        let rib = self.get_rib(rib_id);
        let prefixes = vec![&emp_cpath, &crate_cpath, &rib.cpath];

        // resolve to local variables or parameters
        assert!(path.segments.len() > 0);
        if path.segments.len() == 1 {
            // TODO:
            let mut result = None;
            let ident = &path.segments[0];
            self.resolve_to_local_with_dfs(ident, rib_id, &mut result);
            if result.is_some() {
                return result;
            }
        }

        // absolute path
        if *path.segments.first().unwrap().symbol == "crate" {
            let mut result = None;
            // prefix: ["", "crate"]
            self.resolve_to_item_with_dfs(&prefixes, path, self.crate_rib_id, &mut result);
            result
        }
        // relative path
        else {
            // search in all siblings
            let mut result = None;

            // search from this module (if this rib is not module, starts from its parent module)
            if rib.kind != RibKind::Mod {
                let parent_module_rib = self.get_parent_module(rib_id).unwrap();
                self.resolve_to_item_with_dfs(&prefixes, path, parent_module_rib.id, &mut result);
            } else {
                self.resolve_to_item_with_dfs(&prefixes, path, rib_id, &mut result);
            }

            result
        }
    }

    /// Resovle path to item or module
    fn resolve_to_local_with_dfs(
        &self,
        ident: &Ident,
        rib_id: RibId,
        result: &mut Option<Rc<Binding>>,
    ) {
        let rib = self.get_rib(rib_id);
        if !matches!(rib.kind, RibKind::Block | RibKind::Func) {
            return;
        }

        for (name, binding) in &rib.bindings {
            if matches!(binding.kind, BindingKind::Let | BindingKind::Param)
                && *ident.symbol == **name
            {
                *result = Some(Rc::clone(binding));
                return;
            }
        }

        if let Some(parent) = rib.parent {
            self.resolve_to_local_with_dfs(ident, parent, result);
        }
    }

    fn get_parent_module(&self, rib_id: RibId) -> Option<&Rib> {
        let rib = self.get_rib(rib_id);
        if let Some(parent_rib_id) = rib.parent {
            let parent_rib = self.get_rib(parent_rib_id);
            if parent_rib.kind == RibKind::Mod {
                Some(parent_rib)
            } else {
                self.get_parent_module(parent_rib_id)
            }
        } else {
            None
        }
    }

    /// Resovle path to item or module
    fn resolve_to_item_with_dfs(
        &self,
        prefixes: &[&CanonicalPath],
        path: &Path,
        rib_id: RibId,
        result: &mut Option<Rc<Binding>>,
    ) {
        if result.is_some() {
            return;
        }

        let rib = self.get_rib(rib_id);

        for (_, binding) in &rib.bindings {
            if matches!(binding.kind, BindingKind::Item | BindingKind::Mod) {
                for prefix in prefixes {
                    let path_with_prefix = CanonicalPath::from_path(prefix, path);
                    //eprintln!("compare {:?} with {:?}", &path_with_prefix, binding.cpath);
                    if *binding.cpath == path_with_prefix {
                        *result = Some(Rc::clone(binding));
                        return;
                    }
                }
            }
        }

        for child in &rib.children {
            // TODO: if `pub`
            self.resolve_to_item_with_dfs(prefixes, path, *child, result);
        }
    }
}
