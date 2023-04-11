use std::rc::Rc;

use super::{Binding, BindingKind, Resolver, Rib, RibId};
use crate::{
    ast::{self, Path, StmtKind},
    span::Ident,
};

impl Resolver {
    fn get_current_rib_mut(&mut self) -> &mut Rib {
        let current_rib_id = self.current_ribs.last().unwrap();
        self.interned.get_mut(&current_rib_id).unwrap()
    }

    fn push_rib(&mut self) {
        let parent = self.current_ribs.last().copied();
        let rib = Rib::new(self.get_next_rib_id(), parent, self.current_cpath.clone());
        self.current_ribs.push(rib.id);
        self.interned.insert(rib.id, rib);
    }

    fn pop_rib(&mut self) -> RibId {
        self.current_ribs.pop().unwrap()
    }

    fn get_next_rib_id(&mut self) -> u32 {
        let id = self.next_rib_id;
        self.next_rib_id += 1;
        id
    }

    fn push_segment_to_current_cpath(&mut self, seg: Rc<String>) {
        self.current_cpath.push_seg(seg);
    }

    fn pop_segment_from_current_cpath(&mut self) -> Option<Rc<String>> {
        self.current_cpath.pop_seg()
    }

    fn register_decl_to_current_rib(&mut self, ident: &Ident, kind: BindingKind) {
        self.def_to_rib
            .insert(ident.clone(), *self.current_ribs.last().unwrap());

        let mut cpath = self.current_cpath.clone();
        cpath.push_seg(Rc::clone(&ident.symbol));
        self.get_current_rib_mut().insert_binding(
            Rc::clone(&ident.symbol),
            Binding {
                kind: kind,
                cpath: Rc::new(cpath),
            },
        );
    }

    pub fn register_use_to_current_rib(&mut self, path: &Path) {
        self.use_to_rib
            .insert(path.clone(), *self.current_ribs.last().unwrap());
    }
}

impl<'ctx> ast::visitor::Visitor<'ctx> for Resolver {
    fn visit_crate(&mut self, _krate: &'ctx ast::Crate) {
        // push "crate" to cpath
        self.push_segment_to_current_cpath(Rc::new("crate".to_string()));

        // push new rib
        self.push_rib();
    }

    fn visit_crate_post(&mut self, _krate: &'ctx ast::Crate) {
        // pop "crate" from current cpath
        let krate = self.pop_segment_from_current_cpath().unwrap();

        assert_eq!(*krate, "crate");
        // pop rib
        let krate_rib = self.pop_rib();
        assert_eq!(krate_rib, 0);
    }

    fn visit_module_item(&mut self, module: &'ctx ast::Module) {
        // register cmodule name
        self.register_decl_to_current_rib(&module.name, BindingKind::Mod);

        // push module name to cpath
        self.push_segment_to_current_cpath(Rc::clone(&module.name.symbol));
        // push new rib
        self.push_rib();
    }

    fn visit_module_item_post(&mut self, _module: &'ctx ast::Module) {
        // pop mod name from cpath
        self.pop_segment_from_current_cpath().unwrap();

        // pop current rib
        self.pop_rib();
    }

    fn visit_func(&mut self, func: &'ctx ast::Func) {
        // register func name
        self.register_decl_to_current_rib(&func.name, BindingKind::Item);

        // push func name to cpath
        self.push_segment_to_current_cpath(Rc::clone(&func.name.symbol));

        // push new rib
        self.push_rib();

        // push new rib
        self.push_rib();

        // insert parameters to rib
        for (param, _) in &func.params {
            // register param name
            self.register_decl_to_current_rib(param, BindingKind::Param);
        }
    }

    fn visit_func_post(&mut self, _: &'ctx ast::Func) {
        // pop func name from cpath
        self.pop_segment_from_current_cpath().unwrap();

        // pop current rib
        self.pop_rib(); // func
        self.pop_rib(); // param
    }

    fn visit_struct_item(&mut self, strct: &'ctx ast::StructItem) {
        self.register_decl_to_current_rib(&strct.ident, BindingKind::Item);
    }

    fn visit_block(&mut self, _block: &'ctx ast::Block) {
        // push new rib
        self.push_rib();
    }

    fn visit_block_post(&mut self, _: &'ctx ast::Block) {
        // pop current rib
        self.pop_rib();
    }

    fn visit_stmt(&mut self, stmt: &'ctx ast::Stmt) {
        if let StmtKind::Let(let_stmt) = &stmt.kind {
            // insert local variables
            self.register_decl_to_current_rib(&let_stmt.ident, BindingKind::Let);
        }
    }

    fn visit_path(&mut self, path: &'ctx ast::Path) {
        self.register_use_to_current_rib(path);
    }
}
