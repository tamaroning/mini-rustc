struct S {
    a: i32,
}

fn f(s: S) -> S {
    if s.a == 1 {
        S { a: 10 }
    } else {
        S { a: 20 }
    }
}

fn main() -> i32 {
    f(S { a: 1 }).a
}
