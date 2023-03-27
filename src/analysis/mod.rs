use crate::ast::NodeId;
use crate::ty::Ty;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Ctxt {
    ty_mappings: HashMap<NodeId, Rc<Ty>>,
    pub dump_enabled: bool,
}

impl Ctxt {
    pub fn new(dump_enabled: bool) -> Self {
        Ctxt {
            ty_mappings: HashMap::new(),
            dump_enabled,
        }
    }

    pub fn insert_type(&mut self, node_id: NodeId, ty: Rc<Ty>) {
        self.ty_mappings.insert(node_id, ty);
    }

    pub fn get_type(&self, node_id: NodeId) -> Rc<Ty> {
        Rc::clone(self.ty_mappings.get(&node_id).unwrap())
    }

    pub fn type_info(&self, ty: &Ty) -> TyInfo {
        let size = match ty {
            Ty::I32 => 4,
            Ty::Bool => 1,
            Ty::Never => 0,
            Ty::Unit => 0,
        };
        TyInfo { size }
    }

    pub fn is_zst(&self, ty: &Ty) -> bool {
        self.type_info(ty).size == 0
    }
}

pub struct TyInfo {
    pub size: u32,
}
