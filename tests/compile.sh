#!/bin/bash
cd $(dirname $0)
RUSTC="../target/debug/mini-rustc"
TMP="../tmp.ll"

RED='\033[0;31m'
GREEN='\033[0;32m'
GRAY='\033[0;30m'
NC='\033[0m' # No Color

compile() {
    input="$1"

    rm $TMP
    $RUSTC "$input" >$TMP
    res="$?"

    if [ "$res" = "0" ]; then
        echo -e "[${GREEN}OK${NC}] $input"
    else
        echo - e "[${RED}ERROR${NC}] $input ${GRAY}=> Compile failed${NC}"
        exit 1
    fi
}

QT="'"
NL=$'\n'

echo "===== Compile Tests ====="
compile 'fn main() -> () { }'
# comments
compile "fn main() -> () { // comment${NL}}"
compile "//${NL}fn main() -> () { }"
compile "fn main() -> () { //${NL}}"
# ref type
compile 'fn main() -> i32 { let string: &'$QT'static str; 0  }'
# string literal
compile 'fn main() -> i32 { "Hello"; "World"; 0 }'
compile 'fn main() -> i32 { let s: &'$QT'static str; s = "Hello, World"; 0 }'
# never type
compile 'fn main() -> i32 { return 0; }'
compile 'fn main() -> i32 { return 0 }'
#compile 'fn main() -> () { let a: i32 = (return ()); a = return (); }'
compile 'fn main() -> () { { let never: ! = (return ()); } }'
compile 'fn main() -> () { { let unit: () = (return ()); } }'
compile 'fn main() -> () { let never: ! = (return ()); }'
compile 'fn main() -> () { let unit: () = (return ()); }'
# typeck block expr
compile 'fn main() -> () { { let u: () = { }; } }'
compile 'fn main() -> () { { let u: () = { () }; } }'
compile 'fn main() -> () { { let n: i32 = { true; 2 + 3 }; } }'
compile 'fn main() -> () { { let u: () = { { { }; { ( { () }) } } }; } }'
compile 'fn main() -> () { { let u: () = { {}; {}; {}; }; } }'
compile 'fn main() -> () { { let n: i32 = { { true }; { { { 0 } } } }; } }'
compile 'fn main() -> () { let n: i32; let n: i32 = { { }; n }; }'
# typeck let
compile 'fn main() -> () { { let unit: () = (); } }'
compile 'fn main() -> () { let a: i32 = 1; }'
# typeck func body
compile 'fn main() -> () { () }'
compile 'fn main() -> i32 { 0 }'
# extern
compile 'extern "C" { }
fn main() -> () { }'
compile 'extern "C" { fn f() -> (); fn g() -> (); fn h() -> (); }
fn main() -> () { }'
compile 'extern "C" { fn printf(s: &str) -> i32; }
fn main() -> () { }'
compile 'extern "C" { fn add(a: i32, b: i32) -> i32; fn add3(a: i32, b: i32, c: i32) -> i32; }
fn main() -> () { }'
# func call
compile 'fn take_num(n: i32) -> () { } fn main() -> () { take_num(0); }'
compile 'fn take_two(n: i32, m: i32) -> () { } fn main() -> () { take_two(0, 1,); }'
compile 'fn take(b: bool, n: i32) -> () { } fn main() -> () { take(false, 0); }'
#compile 'fn take(s: &str) -> () { } fn main() -> () { let u: () = take("Hello"); }'
compile 'fn f(n: i32) -> bool { true } fn main() -> () { let b: bool = f(1 + 2 * 3 - 4); }'
# array expr
#compile 'fn main() -> () { [1]; [1, 1]; [1, 2, 3]; [1, 1,]; }'
#compile 'fn main() -> () { [(), (), ()]; [(), (())];  }'
#compile 'fn main() -> () { [[()], [()], [()],];  }'
#compile 'fn main() -> () { [[[0]]]; }'
#compile 'fn main() -> () { [1, return (), ]; }'
# TODO: compile 'fn main() -> () { [()][0] }'
# if
#compile 'fn main() -> () { if (true) { } else { } }'
#compile 'fn main() -> () { if (1 == 1) { } else { } }'
#compile 'fn main() -> () { if (true) { } }'
# mod
compile 'fn main() -> () { } mod emp { }'
compile 'fn main() -> () { } mod a { fn f() -> () { } }'
compile 'fn main() -> () { } mod a { mod b { mod c { } } }'
compile 'fn main() -> () { } mod a { mod a { mod a { } } }'
compile 'fn main() -> () { } mod a { mod a { mod a { fn a() -> () { }  } } }'
# no forward decl
compile 'fn main() -> () { let s: S; } struct S { }'
# absolute path (`crate...`)
compile 'fn f() -> () { } fn main() -> () { crate::f() }'
compile 'mod a { fn f() -> () { } } fn main() -> () { crate::a::f() }'
compile 'mod a { fn f() -> () { } } mod b { fn f() -> () { } } fn main() -> () { crate::b::f() }'
# relative path
compile 'fn f() -> () { } fn main() -> () { crate::f() }'
compile 'mod a { fn f() -> () { } } fn main() -> () { a::f() }'
compile 'mod a { fn f() -> () { } } mod b { fn f() -> () { } } fn main() -> () { b::f() }'
# TODO: typecheck all items first and then typecheck bodies
#compile 'fn main() -> () { f(); } fn f() -> () { }'
# scope
compile 'fn main() -> () { let a: i32 = 0; let b: i32 = { { a } }; }'
