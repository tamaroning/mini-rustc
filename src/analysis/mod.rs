use crate::ast::NodeId;
use crate::ty::Ty;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Ctxt {
    // move to typing context?
    /// Result of typecheck
    ty_mappings: HashMap<NodeId, Rc<Ty>>,
    // move to typing context?
    fn_types: HashMap<String, Rc<Ty>>,
    pub dump_enabled: bool,
}

impl Ctxt {
    pub fn new(dump_enabled: bool) -> Self {
        Ctxt {
            ty_mappings: HashMap::new(),
            fn_types: HashMap::new(),
            dump_enabled,
        }
    }

    pub fn insert_type(&mut self, node_id: NodeId, ty: Rc<Ty>) {
        self.ty_mappings.insert(node_id, ty);
    }

    pub fn get_type(&self, node_id: NodeId) -> Rc<Ty> {
        Rc::clone(self.ty_mappings.get(&node_id).unwrap())
    }

    pub fn lookup_fn_type(&self, func_name: &String) -> Option<Rc<Ty>> {
        self.fn_types.get(func_name).map(Rc::clone)
    }

    pub fn set_fn_type(&mut self, func_name: String, fn_ty: Rc<Ty>) {
        self.fn_types.insert(func_name, fn_ty);
    }

    /*
    pub fn type_info(&self, ty: &Ty) -> TyInfo {
        let size = match ty {
            Ty::I32 => 4,
            Ty::Bool => 1,
            Ty::Never => 0,
            Ty::Unit => 0,
            Ty::Error => unreachable!(),
        };
        TyInfo { size }
    }

    pub fn is_zst(&self, ty: &Ty) -> bool {
        self.type_info(ty).size == 0
    }
    */
}

pub struct TyInfo {
    pub size: u32,
}
