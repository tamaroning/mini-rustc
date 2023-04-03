pub mod ty;

use crate::ast::{self, Crate, NodeId};
use crate::middle::ty::{AdtDef, Ty};
use crate::resolve::Resolver;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Ctxt {
    // TODO: In order to get scopes from variable names during codegen, we need to
    // save information about scopes instead of popping and discarding them during
    // typeck
    // To deal with this, add the following fields to Ctxt
    //  node_id_to_def_id_mappings: HashMap<NodeId, DefId>,
    //  def_id_to_local_info_mappings: HashMap<DefId, LocalInfo>,
    pub resolver: Resolver,
    /// ExprOrStmt to type mappings, which is set by typechecker
    ty_mappings: HashMap<NodeId, Rc<Ty>>,
    // move to tyctxt?
    fn_types: HashMap<String, Rc<Ty>>,
    adt_defs: HashMap<String, AdtDef>,
    pub dump_enabled: bool,
}

impl<'ctx> Ctxt {
    pub fn new(dump_enabled: bool) -> Self {
        Ctxt {
            resolver: Resolver::new(),
            ty_mappings: HashMap::new(),
            fn_types: HashMap::new(),
            adt_defs: HashMap::new(),
            dump_enabled,
        }
    }

    // Resolution Stage

    pub fn resolve(&mut self, krate: &'ctx Crate) {
        ast::visitor::go(&mut self.resolver, krate);
    }

    // Typecheck Stage

    pub fn insert_type(&mut self, node_id: NodeId, ty: Rc<Ty>) {
        self.ty_mappings.insert(node_id, ty);
    }

    pub fn get_type(&self, node_id: NodeId) -> Rc<Ty> {
        Rc::clone(self.ty_mappings.get(&node_id).unwrap())
    }

    /// Get type of block
    pub fn get_block_type(&self, block: &ast::Block) -> Rc<Ty> {
        if let Some(stmt) = block.stmts.last() {
            let last_stmt_ty = &self.get_type(stmt.id);
            Rc::clone(last_stmt_ty)
        } else {
            // no statement. Unit type
            Rc::new(Ty::Unit)
        }
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

    // Codegen stage

    // FIXME: infinite loop in case of recursive struct
    // e.g. `struct S { s: S }`
    pub fn get_size(&self, ty: &Ty) -> u32 {
        match ty {
            Ty::Unit => 8, // TODO: 0
            Ty::Bool => 8, // TODO: 1
            Ty::I32 => 8,  // TODO: 4
            Ty::Str => panic!("ICE"),
            Ty::Array(elem_ty, n) => self.get_size(elem_ty) * n,
            Ty::Fn(_, _) => 8, // = pointer size
            Ty::Adt(name) => {
                let adt = self.lookup_adt_def(name).unwrap();
                self.get_adt_size(adt)
            }
            Ty::Ref(_, _) => 8,
            Ty::Never => 0,
            Ty::Error => panic!("ICE"),
        }
    }

    // FIXME: infinite loop
    pub fn get_adt_size(&self, adt: &AdtDef) -> u32 {
        let mut size = 0;
        for (_, ty) in &adt.fields {
            size += self.get_size(ty);
        }
        size
    }

    /// Gets offset of the given field.
    /// Returns None if the field does not exists on the ADT.
    // TODO: alignment
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

    /// Flatten all fields of ADT. Returns fields with their offset.
    /// e.g. `S2 { u: (), a: bool } S { a: i32, c: S2, u: () }`
    ///     flatten_struct(s) => [ (i32, 0), ((), 4), (bool, 4) ]
    /// TODO: alignment
    pub fn flatten_struct(&self, adt: &AdtDef) -> Vec<(Rc<Ty>, u32)> {
        let mut ofs_and_tys = vec![];
        let mut ofs = 0;
        self.collect_fields(adt, &mut ofs_and_tys, &mut ofs);
        ofs_and_tys
    }

    fn collect_fields(
        &self,
        adt: &AdtDef,
        ofs_and_tys: &mut Vec<(Rc<Ty>, u32)>,
        current_ofs: &mut u32,
    ) {
        for (_, ty) in &adt.fields {
            if let Ty::Adt(name) = &**ty {
                let adt = self.lookup_adt_def(name).unwrap();
                self.collect_fields(adt, ofs_and_tys, current_ofs);
            } else {
                (*ofs_and_tys).push((Rc::clone(ty), *current_ofs));
                *current_ofs += self.get_size(ty);
            }
        }
    }
}
