use std::rc::Rc;

use super::{Res, Resolver, Rib};
use crate::ast::{self, NodeId, StmtKind};

impl Resolver {
    fn get_current_rib_mut(&mut self) -> &mut Rib {
        let current_rib_id = self.current_ribs.last().unwrap();
        self.interned.get_mut(&current_rib_id).unwrap()
    }

    fn push_rib(&mut self, node_id: NodeId) {
        let rib = Rib::new(self.get_next_rib_id(), self.current_cpath.clone());
        self.current_ribs.push(rib.id);
        self.ribs.insert(node_id, rib.id);
        self.interned.insert(rib.id, rib);
    }

    fn pop_rib(&mut self) {
        self.current_ribs.pop().unwrap();
    }

    fn get_next_rib_id(&mut self) -> u32 {
        let id = self.next_rib_id;
        self.next_rib_id += 1;
        id
    }

    fn push_segment_to_current_cpath(&mut self, seg: Rc<String>, res: Res) {
        self.current_cpath.push_segment(seg, res);
    }

    fn pop_segment_from_current_cpath(&mut self) -> Option<Rc<String>> {
        self.current_cpath.pop_segment()
    }

    pub fn set_ribs_to_ident_node(&mut self, ident_node_id: NodeId) {
        // FIXME: To remove this clone, use tree structure
        // and chaege ident_to_ribs: HashMap<NodeId, Vec<RibId>> to HashMap<NodeId, RibId>
        self.ident_to_ribs
            .insert(ident_node_id, self.current_ribs.clone());
    }
}

impl<'ctx> ast::visitor::Visitor<'ctx> for Resolver {
    fn visit_crate(&mut self, krate: &'ctx ast::Crate) {
        // push "crate" to cpath
        self.push_segment_to_current_cpath(Rc::new("crate".to_string()), Res::Crate(krate.id));

        // push new rib
        self.push_rib(krate.id);
    }

    fn visit_crate_post(&mut self, _krate: &'ctx ast::Crate) {
        // pop "crate" from current cpath
        let krate = self.pop_segment_from_current_cpath().unwrap();

        assert_eq!(*krate, "crate");
        // pop rib
        self.pop_rib();
    }

    fn visit_module_item(&mut self, module: &'ctx ast::Module) {
        // push func name to cpath
        self.push_segment_to_current_cpath(
            Rc::clone(&module.name.symbol),
            Res::Func(module.name.id),
        );
        // push new rib
        self.push_rib(module.id);
    }

    fn visit_module_item_post(&mut self, _module: &'ctx ast::Module) {
        // pop mod name from cpath
        self.pop_segment_from_current_cpath().unwrap();

        // pop current rib
        self.pop_rib();
    }

    fn visit_func(&mut self, func: &'ctx ast::Func) {
        // push func name to cpath
        self.push_segment_to_current_cpath(Rc::clone(&func.name.symbol), Res::Func(func.name.id));

        // insert func name
        self.get_current_rib_mut()
            .insert_binding(Rc::clone(&func.name.symbol), Res::Func(func.name.id));

        // push new rib
        self.push_rib(func.id);

        // insert parameters
        for (param, _) in &func.params {
            self.get_current_rib_mut()
                .insert_binding(Rc::clone(&param.symbol), Res::Param(param.id))
        }
    }

    fn visit_func_post(&mut self, _: &'ctx ast::Func) {
        // pop func name from cpath
        self.pop_segment_from_current_cpath().unwrap();

        // pop current rib
        self.pop_rib();
    }

    fn visit_block(&mut self, block: &'ctx ast::Block) {
        // push new rib
        self.push_rib(block.id);
    }

    fn visit_block_post(&mut self, _: &'ctx ast::Block) {
        // pop current rib
        self.pop_rib();
    }

    fn visit_stmt(&mut self, stmt: &'ctx ast::Stmt) {
        if let StmtKind::Let(let_stmt) = &stmt.kind {
            // insert local variables
            self.get_current_rib_mut().insert_binding(
                Rc::clone(&let_stmt.ident.symbol),
                Res::Let(let_stmt.ident.id),
            )
        }
    }

    fn visit_ident(&mut self, ident: &'ctx ast::Ident) {
        self.set_ribs_to_ident_node(ident.id);
    }
}
