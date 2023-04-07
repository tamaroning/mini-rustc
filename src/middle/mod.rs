pub mod ty;

use crate::ast::{self, Crate, NodeId};
use crate::middle::ty::{AdtDef, Ty};
use crate::resolve::Resolver;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Debug)]
pub struct Ctxt {
    pub dump_enabled: bool,
    // Set during name resolution stage
    pub resolver: Resolver,

    // Set during typecheck stage
    /// ExprOrStmtOrBlock to type mappings
    ty_mappings: HashMap<NodeId, Rc<Ty>>,
    fn_types: HashMap<Rc<String>, Rc<Ty>>,
    adt_defs: HashMap<Rc<String>, Rc<AdtDef>>,

    // Set during rvalue anlaysis stage
    /// all node ids of place expressions
    /// ref: https://doc.rust-lang.org/reference/expressions.html?highlight=rvalue#place-expressions-and-value-expressions
    lvalues: HashSet<NodeId>,

    // codegen
    /// cache
    // TODO: remove (x86-64 backend)
    adt_info_cache: HashMap<String, Rc<AdtInfo>>,
}

impl<'ctx> Ctxt {
    pub fn new(dump_enabled: bool) -> Self {
        Ctxt {
            dump_enabled,
            resolver: Resolver::new(),
            ty_mappings: HashMap::new(),
            fn_types: HashMap::new(),
            adt_defs: HashMap::new(),
            lvalues: HashSet::new(),
            adt_info_cache: HashMap::new(),
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

    pub fn lookup_fn_type(&self, func_name: &String) -> Option<Rc<Ty>> {
        self.fn_types.get(func_name).map(Rc::clone)
    }

    pub fn set_fn_type(&mut self, func_name: Rc<String>, fn_ty: Rc<Ty>) {
        self.fn_types.insert(func_name, fn_ty);
    }

    pub fn lookup_adt_def(&self, adt_name: &String) -> Option<Rc<AdtDef>> {
        self.adt_defs.get(adt_name).map(Rc::clone)
    }

    pub fn set_adt_def(&mut self, name: Rc<String>, adt: AdtDef) {
        self.adt_defs.insert(name, Rc::new(adt));
    }

    pub fn get_adt_defs(&self) -> &HashMap<Rc<String>, Rc<AdtDef>> {
        &self.adt_defs
    }

    // Rvalue analysis stage
    pub fn register_lvalue(&mut self, node_id: NodeId) {
        self.lvalues.insert(node_id);
    }

    pub fn is_lvalue(&mut self, node_id: NodeId) -> bool {
        self.lvalues.contains(&node_id)
    }

    // Codegen stage

    // TODO: remove (x86-64 backend)
    /// Get size of ADT from cache.
    /// Calculate ADT info if it does not exist on cache.
    pub fn get_adt_info(&mut self, name: &String) -> Rc<AdtInfo> {
        if let Some(adt_info) = self.adt_info_cache.get(name) {
            Rc::clone(adt_info)
        } else {
            let adt = Rc::clone(self.adt_defs.get(name).unwrap());
            let adt_info = Rc::new(self.calc_adt_info(&adt));
            self.adt_info_cache
                .insert(name.clone(), Rc::clone(&adt_info));
            adt_info
        }
    }

    // TODO: remove (x86-64 backend)
    /// Get size of type
    /// ref: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=3b57a75c2bb154e552a9014f446c1c06
    // FIXME: infinite loop in case of recursive struct
    // e.g. `struct S { s: S }`
    pub fn get_size(&mut self, ty: &Ty) -> usize {
        match ty {
            Ty::Unit => 0,
            Ty::Bool => 1,
            Ty::I32 => 4,
            Ty::Str => panic!("ICE"),
            Ty::Array(elem_ty, n) => self.get_size(elem_ty) * n,
            Ty::Fn(_, _) => 8,
            Ty::Adt(name) => self.get_adt_info(name).size,
            Ty::Ref(_, _) => 8,
            Ty::Never => 0,
            Ty::Error => panic!("ICE"),
        }
    }

    // TODO: remove (x86-64 backend)
    /// Perform actual calculation
    /// TODO: add tests
    fn calc_adt_info(&mut self, adt: &AdtDef) -> AdtInfo {
        let mut field_offsets = HashMap::new();
        let mut field_sizes = HashMap::new();
        let mut max_fd_align = 1;

        let mut current_ofs = 0;
        for (fd, fd_ty) in &adt.fields {
            let fd_size = self.get_size(fd_ty);
            let fd_align = self.get_align(fd_ty);
            max_fd_align = max(max_fd_align, fd_align);

            field_offsets.insert(Rc::clone(fd), current_ofs);
            field_sizes.insert(Rc::clone(fd), fd_size);

            current_ofs += fd_size;
            current_ofs += calc_padding(current_ofs, fd_align);
        }

        current_ofs += calc_padding(current_ofs, max_fd_align);

        AdtInfo {
            size: current_ofs,
            align: max_fd_align,
            field_offsets,
            field_sizes,
        }
    }

    // TODO: remove (x86-64 backend)
    /// Recursively caluculate alignment of type
    pub fn get_align(&self, ty: &Ty) -> usize {
        match ty {
            Ty::Unit => 1,
            Ty::Bool => 1,
            Ty::I32 => 4,
            Ty::Str => panic!("ICE"),
            Ty::Array(elem_ty, _) => self.get_align(elem_ty),
            Ty::Fn(_, _) => 8,
            Ty::Adt(name) => {
                let adt = self.lookup_adt_def(name).unwrap();
                adt.fields
                    .iter()
                    .map(|(_, ty)| self.get_align(ty))
                    .max()
                    .unwrap_or(1)
            }
            Ty::Ref(_, _) => 8,
            Ty::Never => 1,
            Ty::Error => panic!("ICE"),
        }
    }

    // TODO: remove (x86-64 backend)
    /// Gets offset of the given field.
    pub fn get_field_offset(&mut self, adt_name: &String, field: &String) -> Option<usize> {
        let adt_info = self.get_adt_info(adt_name);
        adt_info.field_offsets.get(field).map(|u| *u)
    }

    /*
    /// Flatten all fields of ADT to primitive types (no ADT or array) but ignores ZST fields. Returns fields with their offset.
    /// e.g. `S2 { u: (), a: bool } S { a: i32, b: S2, u: (), c: i32 }`
    ///     flatten_struct(s) => [ (i32, 0), (bool, 4), (i32, 8) ]
    /// TODO: alignment
    pub fn flatten_struct(&mut self, adt: &AdtDef) -> Vec<(Rc<Ty>, usize)> {
        let mut ofs_and_tys = vec![];
        let mut ofs = 0;
        self.collect_fields(adt, &mut ofs_and_tys, &mut ofs);
        ofs_and_tys
    }

    fn collect_fields(
        &mut self,
        adt: &AdtDef,
        ofs_and_tys: &mut Vec<(Rc<Ty>, usize)>,
        current_ofs: &mut usize,
    ) {
        for (_, ty) in &adt.fields {
            if let Ty::Adt(name) = &**ty {
                // adt
                let adt = self.lookup_adt_def(name).unwrap();
                self.collect_fields(&adt, ofs_and_tys, current_ofs);
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

    pub fn flatten_array(&mut self, elem_ty: &Rc<Ty>, elem_num: usize) -> Vec<(Rc<Ty>, usize)> {
        let mut ofs_and_tys = vec![];
        let mut ofs = 0;
        self.collect_elems(elem_ty, elem_num, &mut ofs_and_tys, &mut ofs);
        ofs_and_tys
    }

    fn collect_elems(
        &mut self,
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
                self.collect_fields(&adt, ofs_and_tys, current_ofs);
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
    */
}

// TODO: remove (x86-64 backend)
#[derive(Debug)]
pub struct AdtInfo {
    pub size: usize,
    pub align: usize,
    pub field_offsets: HashMap<Rc<String>, usize>,
    pub field_sizes: HashMap<Rc<String>, usize>,
}

// TODO: remove (x86-64 backend)
fn calc_padding(current_ofs: usize, align: usize) -> usize {
    if current_ofs % align != 0 {
        align - (current_ofs % align)
    } else {
        0
    }
}

/*
#[test]
fn flatten_struct_simple() {
    let mut fields = Vec::new();
    fields.push(("a".to_string(), Rc::new(Ty::I32)));
    fields.push(("b".to_string(), Rc::new(Ty::I32)));
    fields.push(("c".to_string(), Rc::new(Ty::I32)));
    let adt = AdtDef { fields };
    let mut ctx = Ctxt::new(false);
    let flatten = ctx.flatten_struct(&adt);
    assert_eq!(*flatten[0].0, Ty::I32);
    assert_eq!(flatten[0].1, 0);
    assert_eq!(*flatten[1].0, Ty::I32);
    assert_eq!(flatten[1].1, 4);
    assert_eq!(*flatten[2].0, Ty::I32);
    assert_eq!(flatten[2].1, 8);
}

#[test]
fn flatten_struct_align() {
    let mut fields = Vec::new();
    fields.push(("a".to_string(), Rc::new(Ty::I32)));
    fields.push(("b".to_string(), Rc::new(Ty::Bool)));
    fields.push(("c".to_string(), Rc::new(Ty::Bool)));
    fields.push(("d".to_string(), Rc::new(Ty::I32)));
    let adt = AdtDef { fields };
    let mut ctx = Ctxt::new(false);
    let flatten = ctx.flatten_struct(&adt);
    assert_eq!(*flatten[0].0, Ty::I32);
    assert_eq!(flatten[0].1, 0);
    assert_eq!(*flatten[1].0, Ty::Bool);
    assert_eq!(flatten[1].1, 4);
    assert_eq!(*flatten[2].0, Ty::Bool);
    assert_eq!(flatten[2].1, 5);
    assert_eq!(*flatten[3].0, Ty::I32);
    assert_eq!(flatten[3].1, 8);
}

#[test]
fn flatten_array_simple() {
    let mut ctx = Ctxt::new(false);
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
    let mut ctx = Ctxt::new(false);
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
*/
