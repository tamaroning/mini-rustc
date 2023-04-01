use std::rc::Rc;

#[derive(Debug, PartialEq, Eq)]
pub enum Ty {
    Unit,
    Bool,
    I32,
    Str,
    Array(Rc<Ty>, u32),
    Fn(Vec<Rc<Ty>>, Rc<Ty>),
    Adt(String),
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

    pub fn is_adt(&self) -> bool {
        matches!(self, Ty::Adt(_))
    }
}

#[derive(Debug)]
pub struct AdtDef {
    pub fields: Vec<(String, Rc<Ty>)>,
}
