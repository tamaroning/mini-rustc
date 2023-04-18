use std::{collections::HashMap, rc::Rc};

use super::{Binding, BindingKind, ResolvedOrRib, Resolver, Rib, RibId, RibKind};
use crate::{
    ast::{self, Path, StmtKind},
    span::Ident,
};

impl Resolver {
    fn get_current_rib_mut(&mut self) -> &mut Rib {
        let current_rib_id = self.current_ribs.last().unwrap();
        self.interned.get_mut(&current_rib_id).unwrap()
    }

    fn push_rib(&mut self, kind: RibKind) {
        let new_rib_id = self.get_next_rib_id();
        let parent_rib_id = self.current_ribs.last().copied();
        if let Some(parent_rib_id) = parent_rib_id {
            let parent_rib = self.get_rib_mut(parent_rib_id);
            parent_rib.children.push(new_rib_id);
        }
        let rib = Rib::new(new_rib_id, kind, parent_rib_id, self.current_cpath.clone());
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

    fn push_variable_scope(&mut self) {
        self.current_variable_scopes.push(HashMap::new());
    }

    fn pop_variable_scope(&mut self) {
        self.current_variable_scopes.pop();
    }

    fn get_current_scope_mut(&mut self) -> Option<&mut HashMap<Rc<String>, Rc<Binding>>> {
        self.current_variable_scopes.last_mut()
    }

    fn get_current_scopes(&self) -> &[HashMap<Rc<String>, Rc<Binding>>] {
        &self.current_variable_scopes
    }

    fn insert_item_def(&mut self, ident: &Ident, kind: BindingKind) {
        self.item_def_to_rib
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

    fn get_num_of_same_variable_name_in_scopes(&self, ident: &Ident) -> u32 {
        let mut res = 0;
        for scope in self.get_current_scopes() {
            if let Some(bind) = scope.get(&ident.symbol)
                && matches!(bind.kind, BindingKind::Let(_) | BindingKind::Param){
                res+=1;
            }
        }
        res
    }

    fn insert_var_decl(&mut self, ident: &Ident, kind: BindingKind) {
        let mut cpath = self.current_cpath.clone();
        cpath.push_seg(Rc::clone(&ident.symbol));
        let binding = Binding {
            kind,
            cpath: Rc::new(cpath),
        };
        let binding = Rc::new(binding);
        self.var_decl_to_res
            .insert(ident.clone(), Rc::clone(&binding));

        self.get_current_scope_mut()
            .unwrap()
            .insert(Rc::clone(&ident.symbol), binding);
    }

    fn find_variable_in_scope(&self, path: &Path) -> Option<Rc<Binding>> {
        if path.segments.is_empty() || path.segments.len() > 1 {
            return None;
        }
        let ident = &path.segments[0];
        // search path from the current scope to the old scope
        for scope in self.get_current_scopes().iter().rev() {
            if let Some(binding) = scope.get(&ident.symbol) {
                return Some(Rc::clone(binding));
            }
        }
        None
    }

    fn insert_use_of_variable(&mut self, path: &Path, binding: Rc<Binding>) {
        self.path_use_to_rib
            .insert(path.clone(), ResolvedOrRib::Resolved(binding));
    }

    fn insert_use_of_item(&mut self, path: &Path) {
        self.path_use_to_rib.insert(
            path.clone(),
            ResolvedOrRib::UnResolved(*self.current_ribs.last().unwrap()),
        );
    }
}

impl<'ctx> ast::visitor::Visitor<'ctx> for Resolver {
    fn visit_crate(&mut self, _krate: &'ctx ast::Crate) {
        // push "crate" to cpath
        self.push_segment_to_current_cpath(Rc::new("crate".to_string()));

        // push new rib
        self.push_rib(RibKind::Mod);

        self.crate_rib_id = self.get_current_rib_mut().id;
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
        self.insert_item_def(&module.name, BindingKind::Mod);

        // push module name to cpath
        self.push_segment_to_current_cpath(Rc::clone(&module.name.symbol));
        // push new rib
        self.push_rib(RibKind::Mod);
    }

    fn visit_module_item_post(&mut self, _module: &'ctx ast::Module) {
        // pop mod name from cpath
        self.pop_segment_from_current_cpath().unwrap();

        // pop current rib
        self.pop_rib();
    }

    fn visit_func(&mut self, func: &'ctx ast::Func) {
        // register func name
        self.insert_item_def(&func.name, BindingKind::Item);

        // push func name to cpath
        self.push_segment_to_current_cpath(Rc::clone(&func.name.symbol));

        // push new rib
        self.push_rib(RibKind::Func);

        // push variable scope
        self.push_variable_scope();

        // insert parameters to rib
        for (param, _) in &func.params {
            // register param name
            self.insert_var_decl(param, BindingKind::Param);
        }
    }

    fn visit_func_post(&mut self, _: &'ctx ast::Func) {
        // pop func name from cpath
        self.pop_segment_from_current_cpath().unwrap();

        // pop current rib
        self.pop_rib(); // func

        // pop varible scope
        self.pop_variable_scope();
    }

    fn visit_struct_item(&mut self, strct: &'ctx ast::StructItem) {
        self.insert_item_def(&strct.ident, BindingKind::Item);
    }

    fn visit_block(&mut self, _block: &'ctx ast::Block) {
        // push new rib
        self.push_rib(RibKind::Block);

        // push variable scope
        self.push_variable_scope();
    }

    fn visit_block_post(&mut self, _: &'ctx ast::Block) {
        // pop current rib
        self.pop_rib();

        // pop varible scope
        self.pop_variable_scope();
    }

    fn visit_stmt_post(&mut self, stmt: &'ctx ast::Stmt) {
        if let StmtKind::Let(let_stmt) = &stmt.kind {
            // insert local variables
            let shadowing_index = self.get_num_of_same_variable_name_in_scopes(&let_stmt.ident);
            self.insert_var_decl(&let_stmt.ident, BindingKind::Let(shadowing_index));
        }
    }

    fn visit_path(&mut self, path: &'ctx Path) {
        // try to resolve path to local variables
        if let Some(binding) = self.find_variable_in_scope(path) {
            self.insert_use_of_variable(path, binding)
        } else {
            self.insert_use_of_item(path);
        }
    }
}
