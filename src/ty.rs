#[derive(Debug, PartialEq, Eq)]
pub enum Ty {
    Unit,
    Never,
    I32,
    Bool,
    Error,
}
