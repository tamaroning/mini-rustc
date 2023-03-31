use std::rc::Rc;

#[derive(Debug, PartialEq, Eq)]
pub enum Ty {
    Unit,
    Bool,
    I32,
    Array(Rc<Ty>, u32),
    Fn(Vec<Rc<Ty>>, Rc<Ty>),
    Adt(String),
    Never,
    Error,
}

impl Ty {
    pub fn get_adt_name(&self) -> Option<&String> {
        if let Ty::Adt(name) = self {
            Some(name)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct AdtDef {
    pub fields: Vec<(String, Rc<Ty>)>,
}
