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

    /// Get size of type
    /// ref: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=3b57a75c2bb154e552a9014f446c1c06
    // FIXME: infinite loop in case of recursive struct
    // e.g. `struct S { s: S }`
    pub fn get_size(&self, ty: &Ty) -> usize {
        match ty {
            Ty::Unit => 0,
            Ty::Bool => 1,
            Ty::I32 => 4,
            Ty::Str => panic!("ICE"),
            Ty::Array(elem_ty, n) => self.get_size(elem_ty) * n,
            Ty::Fn(_, _) => 8, // = pointer size FIXME: correct?
            Ty::Adt(name) => {
                let adt = self.lookup_adt_def(name).unwrap();
                self.get_adt_size(adt)
            }
            Ty::Ref(_, _) => 8, // TODO: 4
            Ty::Never => 0,
            Ty::Error => panic!("ICE"),
        }
    }

    // FIXME: infinite loop
    pub fn get_adt_size(&self, adt: &AdtDef) -> usize {
        let mut size = 0;
        for (_, ty) in &adt.fields {
            size += self.get_size(ty);
        }
        size
    }

    pub fn get_align(&self, ty: &Ty) -> usize {
        match ty {
            Ty::Unit => 1,
            Ty::Bool => 1,
            Ty::I32 => 4,
            Ty::Str => panic!("ICE"),
            Ty::Array(elem_ty, _) => self.get_align(elem_ty),
            Ty::Fn(_, _) => 8,
            Ty::Adt(_) => {
                let size = self.get_size(ty);
                if size == 0 {
                    1
                } else {
                    size
                }
            }
            Ty::Ref(_, _) => 8,
            Ty::Never => 1,
            Ty::Error => panic!("ICE"),
        }
    }

    /// Gets offset of the given field.
    /// Returns None if the field does not exists on the ADT.
    // TODO: alignment
    // TODO: add tests
    pub fn get_field_offsett(&self, adt: &AdtDef, f: &String) -> Option<usize> {
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

    /// Flatten all fields of ADT to primitive types (no ADT or array) but ignores ZST fields. Returns fields with their offset.
    /// e.g. `S2 { u: (), a: bool } S { a: i32, c: S2, u: () }`
    ///     flatten_struct(s) => [ (i32, 0), ((), 4), (bool, 4) ]
    /// TODO: alignment
    pub fn flatten_struct(&self, adt: &AdtDef) -> Vec<(Rc<Ty>, usize)> {
        let mut ofs_and_tys = vec![];
        let mut ofs = 0;
        self.collect_fields(adt, &mut ofs_and_tys, &mut ofs);
        ofs_and_tys
    }

    fn collect_fields(
        &self,
        adt: &AdtDef,
        ofs_and_tys: &mut Vec<(Rc<Ty>, usize)>,
        current_ofs: &mut usize,
    ) {
        for (_, ty) in &adt.fields {
            if let Ty::Adt(name) = &**ty {
                // adt
                let adt = self.lookup_adt_def(name).unwrap();
                self.collect_fields(adt, ofs_and_tys, current_ofs);
            } else if let Ty::Array(elem_ty, elem_num) = &**ty {
                // array
                self.collect_elems(elem_ty, *elem_num, ofs_and_tys, current_ofs);
            } else if self.get_size(ty) == 0 {
                // ignore ZST
            } else {
                // primitive
                ofs_and_tys.push((Rc::clone(ty), *current_ofs));
                *current_ofs += self.get_size(ty);
            }
        }
    }

    pub fn flatten_array(&self, elem_ty: &Rc<Ty>, elem_num: usize) -> Vec<(Rc<Ty>, usize)> {
        let mut ofs_and_tys = vec![];
        let mut ofs = 0;
        self.collect_elems(elem_ty, elem_num, &mut ofs_and_tys, &mut ofs);
        ofs_and_tys
    }

    fn collect_elems(
        &self,
        elem_ty: &Rc<Ty>,
        n: usize,
        ofs_and_tys: &mut Vec<(Rc<Ty>, usize)>,
        current_ofs: &mut usize,
    ) {
        // ignore ZST
        let elem_size = self.get_size(elem_ty);
        if elem_size == 0 {
            return;
        }

        // FIXME: might be super slow
        for _ in 0..n {
            if let Ty::Adt(name) = &**elem_ty {
                // adt
                let adt = self.lookup_adt_def(name).unwrap();
                self.collect_fields(adt, ofs_and_tys, current_ofs);
            } else if let Ty::Array(elem_elem_ty, elem_elem_num) = &**elem_ty {
                // array
                self.collect_elems(elem_elem_ty, *elem_elem_num, ofs_and_tys, current_ofs)
            } else {
                // primitive
                ofs_and_tys.push((Rc::clone(elem_ty), *current_ofs));
                *current_ofs += self.get_size(elem_ty);
            }
        }
    }
}

#[test]
fn flatten_struct_simple() {
    let mut fields = Vec::new();
    fields.push(("a".to_string(), Rc::new(Ty::I32)));
    fields.push(("b".to_string(), Rc::new(Ty::I32)));
    fields.push(("c".to_string(), Rc::new(Ty::I32)));
    let adt = AdtDef { fields };
    let ctx = Ctxt::new(false);
    let flatten = ctx.flatten_struct(&adt);
    assert_eq!(*flatten[0].0, Ty::I32);
    assert_eq!(flatten[0].1, 0);
    assert_eq!(*flatten[1].0, Ty::I32);
    assert_eq!(flatten[1].1, 4);
    assert_eq!(*flatten[2].0, Ty::I32);
    assert_eq!(flatten[2].1, 8);
}

#[test]
fn flatten_array_simple() {
    let ctx = Ctxt::new(false);
    let elem_ty = Rc::new(Ty::I32);
    let flatten = ctx.flatten_array(&elem_ty, 5);
    dbg!(&flatten);
    assert_eq!(*flatten[0].0, Ty::I32);
    assert_eq!(flatten[0].1, 0);
    assert_eq!(*flatten[1].0, Ty::I32);
    assert_eq!(flatten[1].1, 4);
    assert_eq!(*flatten[2].0, Ty::I32);
    assert_eq!(flatten[2].1, 8);
    assert_eq!(*flatten[3].0, Ty::I32);
    assert_eq!(flatten[3].1, 12);
    assert_eq!(*flatten[4].0, Ty::I32);
    assert_eq!(flatten[4].1, 16);
}

#[test]
fn flatten_struct_containing_array() {
    let mut fields = Vec::new();
    fields.push(("a".to_string(), Rc::new(Ty::I32)));
    fields.push(("b".to_string(), Rc::new(Ty::I32)));
    fields.push(("c".to_string(), Rc::new(Ty::Array(Rc::new(Ty::Bool), 4))));
    fields.push(("d".to_string(), Rc::new(Ty::I32)));
    let adt = AdtDef { fields };
    let ctx = Ctxt::new(false);
    let flatten = ctx.flatten_struct(&adt);
    dbg!(&flatten);
    assert_eq!(flatten.len(), 7);
    assert_eq!(*flatten[0].0, Ty::I32);
    assert_eq!(flatten[0].1, 0);
    assert_eq!(*flatten[1].0, Ty::I32);
    assert_eq!(flatten[1].1, 4);

    assert_eq!(*flatten[2].0, Ty::Bool);
    assert_eq!(flatten[2].1, 8);
    assert_eq!(*flatten[3].0, Ty::Bool);
    assert_eq!(flatten[3].1, 9);
    assert_eq!(*flatten[4].0, Ty::Bool);
    assert_eq!(flatten[4].1, 10);
    assert_eq!(*flatten[5].0, Ty::Bool);
    assert_eq!(flatten[5].1, 11);

    assert_eq!(*flatten[6].0, Ty::I32);
    assert_eq!(flatten[6].1, 12);
}
