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
    pub fn get_size(&self) -> u32 {
        match &self {
            Ty::Unit => 8, // TODO: 0
            Ty::Bool => 8, // TODO: 1
            Ty::I32 => 8,  // TODO: 4
            Ty::Array(elem_ty, n) => elem_ty.get_size() * n,
            Ty::Fn(_, _) => 8, // = pointer size
            Ty::Adt(_) => todo!(),
            Ty::Never => 0,
            Ty::Error => panic!("ICE"),
        }
    }
    /*
    pub fn get_elem_type(&self) -> Option<Rc<Ty>> {
        if let Ty::Array(elem_ty, _) = &self {
            Some(Rc::clone(elem_ty))
        } else {
            None
        }
    }
    */
}
