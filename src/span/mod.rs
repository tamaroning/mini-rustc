use std::cmp::{max, min};
use std::rc::Rc;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Span {
    lo: usize,
    hi: usize,
    src: Rc<String>,
}

impl Span {
    pub fn new(lo: usize, hi: usize, src: Rc<String>) -> Self {
        Span { lo, hi, src }
    }

    pub fn to_snippet(&self) -> String {
        assert!(self.lo <= self.hi);
        assert!(self.hi <= self.src.len());
        let src = &*self.src;
        let s = &src[self.lo()..self.hi()];
        // compress whitespaces
        // FIXME: dirty
        s.replace('\n', " ")
            .replace(['\r', '\t'], "")
            .replace("  ", " ")
            .replace("  ", " ")
            .replace("  ", " ")
            .replace("  ", " ")
            .replace("  ", " ")
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
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.lo, self.hi)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    // TODO: remove symbol and span
    // add ident: crate::span::Ident
    pub symbol: Rc<String>,
    pub span: Span,
}

impl std::fmt::Debug for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\" ({:?})", self.symbol, self.span)
    }
}
