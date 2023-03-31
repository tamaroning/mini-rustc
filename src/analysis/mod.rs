use crate::ast::NodeId;
use crate::ty::{AdtDef, Ty};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Ctxt {
    // move to typing context?
    /// Result of typecheck
    ty_mappings: HashMap<NodeId, Rc<Ty>>,
    // move to typing context?
    fn_types: HashMap<String, Rc<Ty>>,
    adt_defs: HashMap<String, AdtDef>,
    pub dump_enabled: bool,
}

impl Ctxt {
    pub fn new(dump_enabled: bool) -> Self {
        Ctxt {
            ty_mappings: HashMap::new(),
            fn_types: HashMap::new(),
            adt_defs: HashMap::new(),
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

    pub fn lookup_adt_def(&self, adt_name: &String) -> Option<&AdtDef> {
        self.adt_defs.get(adt_name)
    }

    pub fn set_adt_def(&mut self, name: String, adt: AdtDef) {
        self.adt_defs.insert(name, adt);
    }

    pub fn get_size(&self, ty: &Ty) -> u32 {
        match ty {
            Ty::Unit => 8, // TODO: 0
            Ty::Bool => 8, // TODO: 1
            Ty::I32 => 8,  // TODO: 4
            Ty::Array(elem_ty, n) => self.get_size(elem_ty) * n,
            Ty::Fn(_, _) => 8, // = pointer size
            Ty::Adt(name) => {
                let mut size = 0;
                let adt = self.lookup_adt_def(name).unwrap();
                for (_, ty) in &adt.fields {
                    size += self.get_size(ty);
                }
                size
            }
            Ty::Never => 0,
            Ty::Error => panic!("ICE"),
        }
    }

    pub fn get_field_offsett(&self, adt: &AdtDef, f: &String) -> Option<u32> {
        let mut saw_field = false;
        let mut offs = 0;
        for (field, ty) in &adt.fields {
            if f == field {
                saw_field = true;
                break;
            }
            offs += self.get_size(ty);
        }
        if saw_field {
            Some(offs)
        } else {
            None
        }
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

/*
pub struct TyInfo {
    pub size: u32,
}
*/
