use std::rc::Rc;

#[derive(Debug, PartialEq, Eq)]
pub enum Ty {
    Unit,
    Never,
    I32,
    Bool,
    Fn(Vec<Rc<Ty>>, Rc<Ty>),
    Error,
}
