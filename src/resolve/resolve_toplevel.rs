use super::{Binding, BindingKind, CanonicalPath};
use crate::ast::{self, NodeId};
use std::{collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct ResolveTopLevel {
    current_module: Vec<NodeId>,
    current_cpath: CanonicalPath,

    res_bindings: HashMap<NodeId, Rc<Binding>>,
    module_to_children: HashMap<NodeId, Vec<NodeId>>,
    child_to_module: HashMap<NodeId, NodeId>,
}

impl ResolveTopLevel {
    pub fn new() -> Self {
        ResolveTopLevel {
            current_cpath: CanonicalPath::empty(),
            res_bindings: HashMap::new(),
            current_module: vec![],

            // private
            module_to_children: HashMap::new(),
            child_to_module: HashMap::new(),
        }
    }

    pub fn go(&mut self, krate: &ast::Crate) {
        ast::visitor::go(self, krate);
    }

    fn current_module(&self) -> NodeId {
        *self.current_module.last().unwrap()
    }

    fn push_module(&mut self, module_id: NodeId) {
        self.current_module.push(module_id);
        self.module_to_children
            .insert(self.current_module(), vec![]);
    }

    fn pop_module(&mut self) {
        self.current_module.pop();
    }

    fn insert_child(&mut self, child_item: NodeId) {
        self.module_to_children
            .get_mut(&self.current_module())
            .unwrap()
            .push(child_item);
        self.child_to_module
            .insert(child_item, self.current_module());
    }

    // TODO: refine
    pub fn search_ident(&self, sym: &Rc<String>) -> Option<Rc<Binding>> {
        self.res_bindings.values().find_map(|b| {
            if b.cpath.segments.last().unwrap() == sym {
                Some(Rc::clone(&b))
            } else {
                None
            }
        })
    }
}

impl<'ctx> ast::visitor::Visitor<'ctx> for ResolveTopLevel {
    fn visit_crate(&mut self, krate: &'ctx ast::Crate) {
        self.push_module(krate.id);

        self.current_cpath.push_seg(Rc::new("crate".to_string()));
        self.res_bindings.insert(
            krate.id,
            Rc::new(Binding {
                cpath: self.current_cpath.clone(),
                kind: BindingKind::Crate,
            }),
        );
    }

    fn visit_crate_post(&mut self, _krate: &'ctx ast::Crate) {
        self.pop_module();
        assert_eq!(self.current_module, vec![]);
        let k = self.current_cpath.pop_seg();
        assert_eq!(*k.unwrap(), "crate");
    }

    fn visit_module_item(&mut self, module: &'ctx ast::Module) {
        self.insert_child(module.id);
        self.push_module(module.id);

        self.current_cpath.push_seg(Rc::clone(&module.name.symbol));
        self.res_bindings.insert(
            module.id,
            Rc::new(Binding {
                cpath: self.current_cpath.clone(),
                kind: BindingKind::Mod,
            }),
        );
    }

    fn visit_module_item_post(&mut self, _module: &'ctx ast::Module) {
        self.pop_module();
        self.current_cpath.pop_seg().unwrap();
    }

    fn visit_func(&mut self, func: &'ctx ast::Func) {
        self.current_cpath.push_seg(Rc::clone(&func.name.symbol));
        self.res_bindings.insert(
            func.id,
            Rc::new(Binding {
                cpath: self.current_cpath.clone(),
                kind: BindingKind::Func,
            }),
        );
        self.insert_child(func.id);
        self.current_cpath.pop_seg().unwrap();
    }

    fn visit_struct_item(&mut self, strct: &'ctx ast::StructItem) {
        self.current_cpath.push_seg(Rc::clone(&strct.ident.symbol));
        self.res_bindings.insert(
            strct.id,
            Rc::new(Binding {
                cpath: self.current_cpath.clone(),
                kind: BindingKind::Struct,
            }),
        );
        self.insert_child(strct.id);
        self.current_cpath.pop_seg().unwrap();
    }
}
