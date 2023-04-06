use std::rc::Rc;

#[derive(Debug, PartialEq, Eq)]
pub enum Ty {
    Unit,
    Bool,
    I32,
    Str,
    Array(Rc<Ty>, usize),
    Fn(Vec<Rc<Ty>>, Rc<Ty>),
    Adt(Rc<String>),
    Ref(Region, Rc<Ty>),
    Never,
    Error,
}

pub type Region = String;

impl Ty {
    pub fn get_adt_name(&self) -> Option<&String> {
        if let Ty::Adt(name) = self {
            Some(name)
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
        matches!(self, Ty::Never)
    }
}

#[derive(Debug)]
pub struct AdtDef {
    pub fields: Vec<(Rc<String>, Rc<Ty>)>,
}
