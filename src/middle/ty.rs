use std::rc::Rc;

use crate::resolve::CanonicalPath;

#[derive(PartialEq, Eq)]
pub struct Ty {
    pub kind: TyKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TyKind {
    Unit,
    Bool,
    I32,
    Str,
    Array(Rc<Ty>, usize),
    Fn(Rc<Vec<Rc<Ty>>>, Rc<Ty>),
    Adt(Rc<CanonicalPath>),
    Ref(Rc<Ty>),
    Never,
    Error,
}

impl Ty {
    pub fn new(kind: TyKind) -> Self {
        Ty { kind }
    }

    pub fn get_adt_name(&self) -> Option<&Rc<CanonicalPath>> {
        if let TyKind::Adt(name) = &self.kind {
            Some(name)
        } else {
            None
        }
    }

    pub fn unit() -> Self {
        Ty { kind: TyKind::Unit }
    }

    pub fn never() -> Self {
        Ty {
            kind: TyKind::Never,
        }
    }

    pub fn error() -> Self {
        Ty {
            kind: TyKind::Error,
        }
    }

    pub fn get_func_type(&self) -> Option<(Rc<Vec<Rc<Ty>>>, Rc<Ty>)> {
        if let TyKind::Fn(params, ret) = &self.kind {
            Some((Rc::clone(params), Rc::clone(ret)))
        } else {
            None
        }
    }

    /*
    pub fn is_adt(&self) -> bool {
        matches!(self, Ty::Adt(_))
    }
    */

    pub fn is_never(&self) -> bool {
        matches!(&self.kind, TyKind::Never)
    }
}

#[derive(Debug)]
pub struct AdtDef {
    pub fields: Vec<(Rc<String>, Rc<Ty>)>,
}

impl std::fmt::Debug for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}
