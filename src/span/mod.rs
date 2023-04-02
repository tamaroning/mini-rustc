use std::cmp::{max, min};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Span {
    lo: usize,
    hi: usize,
    src: Rc<String>,
}

impl Span {
    pub fn new(lo: usize, hi: usize, src: Rc<String>) -> Self {
        Span { lo, hi, src }
    }
    pub fn to_snippet(&self) -> &str {
        assert!(self.lo <= self.hi);
        assert!(self.hi <= self.src.len());
        let src = &*self.src;
        let s = &src[self.lo()..self.hi()];
        s
    }

    pub fn concat(&self, span: &Span) -> Span {
        Span {
            lo: min(self.lo, span.lo),
            hi: max(self.hi, span.hi),
            src: Rc::clone(&self.src),
        }
    }

    pub fn lo(&self) -> usize {
        self.lo
    }

    pub fn hi(&self) -> usize {
        self.hi
    }
    
    /*
    pub fn src(&self) -> Rc<String> {
        Rc::clone(&self.src)
    }
    */
}
