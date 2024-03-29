#!/bin/bash
cd $(dirname $0)
RUSTC="../target/debug/mini-rustc"
TMP="../tmp.ll"
ASM="../tmp.s"
EXE="../tmp"
LLC="llc"
CC="gcc"

RED='\033[0;31m'
GREEN='\033[0;32m'
GRAY='\033[0;30m'
NC='\033[0m' # No Color

assert() {
    expected="$1"
    input="$2"

    rm $TMP $EXE
    $RUSTC "$input" >$TMP
    $LLC -o $ASM $TMP -opaque-pointers
    $CC -o $EXE $ASM
    chmod +x $EXE
    $EXE
    actual="$?"

    if [ "$actual" = "$expected" ]; then
        echo -e "[${GREEN}OK${NC}] $input ${GRAY}=> $actual${NC}"
    else
        echo -e "[${RED}ERROR${NC}] $input ${GRAY}=> $expected expected, but got $actual${NC}"
        exit 1
    fi
}

QT="'"

echo "===== Execute Tests ====="
# return
assert 0 'fn main() -> i32 { return 0; }'
assert 0 'fn main() -> i32 { { return 0; } }'
# func body
assert 0 'fn main() -> i32 { 0 }'
assert 0 'fn main() -> i32 { { { 0 } } }'
assert 10 'fn main() -> i32 { { { 10 } } }'
# unary
# Linux only?
assert 255 'fn main() -> i32 { -1 }'
assert 254 'fn main() -> i32 { -2 }'
# numerical literals
assert 200 'fn main() -> i32 { 100; 200 }'
assert 3 'fn main() -> i32 { 0; 1; 2; 3 }'
# boolean literals
assert 0 'fn main() -> i32 { true; 0 }'
# binop
assert 9 'fn main() -> i32 { 4 + 5 }'
assert 3 'fn main() -> i32 { 10 - 7 }'
assert 6 'fn main() -> i32 { 2 * 3 }'
assert 9 'fn main() -> i32 { 11 + 8 * 2 - 3 * (1 + 5) }'
# let
assert 0 'fn main() -> i32 { let a: i32; let b: i32; 0 }'
assert 0 'fn main() -> i32 { let a: i32 = 0; let b: i32; a }'
assert 7 'fn main() -> i32 { let a: i32 = 4; let b: i32 = a + 3; b }'
# assign
assert 0 'fn main() -> i32 { let a: i32; a = 1; 0 }'
# load
assert 1 'fn main() -> i32 { let a: i32; a = 1; a }'
# func call
assert 0 'fn zero() -> i32 { 0 } fn main() -> i32 { zero() }'
assert 0 'fn id(n: i32) -> i32 { n } fn main() -> i32 { id(0) }'
assert 1 'fn id(n: i32) -> i32 { n } fn main() -> i32 { id(1) }'
# array
assert 0 'fn main() -> i32 { let arr: [i32; 10]; 0 }'
assert 0 'fn main() -> i32 { let arr: [[i32; 4]; 8]; 0 }'
assert 5 'fn main() -> i32 { let arr: [i32; 8]; arr[1] = 5; arr[1] }'
assert 10 'fn main() -> i32 { let arr: [[i32; 4]; 8]; arr[7][3] = 10; arr[7][3] }'
# unit
assert 0 'fn main() -> i32 { (); 0 }'
assert 100 'fn main() -> i32 { (); (); 100 }'
assert 0 'fn main() -> i32 { let u: () = (); 0 }'
assert 0 'fn ret_unit() -> () { return (); } fn main() -> i32 { ret_unit(); 0 }'
assert 0 'fn ret_unit() -> () { () } fn main() -> i32 { ret_unit(); 0 }'
assert 0 'fn ret_unit() -> () { {} } fn main() -> i32 { ret_unit(); 0 }'
# struct
assert 0 'struct Empty { } fn main() -> i32 { let s: Empty; 0 }'
assert 0 'struct Point { x: i32, y: i32, } fn main() -> i32 { let p: Point; 0 }'
assert 0 'struct Pt { x: i32, y: i32 } fn main() -> i32 { let p: Pt; p.x = 1; 0 }'
assert 1 'struct Pt { x: i32, y: i32 } fn main() -> i32 { let p: Pt; p.x = 1; p.x }'
assert 3 'struct Pt { x: i32, y: i32 } fn main() -> i32 { let p: Pt; p.y = 5; p.x = 2; p.y - p.x }'
assert 46 'struct Point { x: i32, y: i32 }
struct Line { p1: Point, p2: Point, }
fn main() -> i32 { let l: Line; l.p1.x = 2; l.p1.y = 3; l.p2.x = 4; l.p2.y = 10; l.p1.x * l.p1.y + l.p2.x * l.p2.y }'
# parameter passing
assert 1 'struct Pt { x: i32, y: i32 } fn x(p: Pt) -> i32 { p.x } fn main() -> i32 { let p: Pt; p.x = 1; x(p) }'
assert 200 'fn fourth(l: [i32; 10]) -> i32 { l[4] } fn main() -> i32 { let arr: [i32; 10]; arr[4] = 200; fourth(arr) }'
assert 0 'fn unit(u: ()) -> () { } fn main() -> i32 { let _: () = unit(()); 0 }'
# struct expression
assert 100 'struct Point { x: i32, y: i32 }
fn main() -> i32 { let p: Point = Point { x: 100, y: 200 }; p.x }'
assert 3 'struct Point { x: i32, y: i32 }
struct Line { p1: Point, p2: Point, }
fn main() -> i32 { let l: Line = Line { p1: Point { x: 1, y:2 }, p2: Point { x: 3, y: 4 } }; Point { x: 5, y: 6 }; l.p2.x }'
assert 200 'struct Point { x: i32, y: i32, z: i32 }
fn main() -> i32 { Point { x: 100, y: 200, z: 300 }.y }'
assert 3 'struct Point { x: i32, y: i32 }
struct Line { p1: Point, p2: Point, }
fn main() -> i32 { Line { p1: Point { x: 1, y:2 }, p2: Point { x: 3, y: 4 } }.p2.x }'
# memcpy
assert 5 'struct Point { x: i32, y: i32 }
fn main() -> i32 { let p1: Point = Point { x: 5, y: 4 }; let p2: Point; p2 = p1; p2.x }'
# shadowing
assert 1 'fn main() -> i32 { let a: i32 = 0; let a: i32 = 1; a }'
assert 2 'fn main() -> i32 { let a: i32 = 0; { let a: i32 = 1; let a: i32 = 2; a } }'
assert 4 'fn main() -> i32 { let a: i32 = 1; let a: i32 = a + a; a + a }'
assert 0 'fn f(a: i32) -> i32 { let a: i32 = a; let a: i32 = a; a } fn main() -> i32 { f(0)  }'
assert 10 'struct S { a: i32 } fn main() -> i32 { let a: i32 = 4; let a: S = S { a: 10 }; a.a }'
# if
assert 1 'fn main() -> i32 { if true { 1 } else { 0 }  }'
assert 0 'fn main() -> i32 { if false { 1 } else { 0 }  }'
assert 0 'struct S { a: i32 }
fn main() -> i32 { let s: S; s = if true { S { a: 0 } } else { S { a: 1 } }; s.a }'
assert 1 'struct S { a: i32 }
fn main() -> i32 { let s: S; s = if false { S { a: 0 } } else { S { a: 1 } }; s.a }'
assert 30 'fn main() -> i32 { let a: i32 = 3; if a == 1 { 10 } else if a == 2 { 20 } else { 30 } }'
assert 20 'fn main() -> i32 { let a: i32 = 2; if a == 1 { 10 } else if a == 2 { 20 } else { 30 } }'
# return struct
assert 0 'struct S { a: i32 } fn f() -> S { S { a: 0 } } fn main() -> i32 { f().a }'
assert 42 'struct S { a: i32 } fn f() -> S { S { a: 42 } } fn main() -> i32 { f().a }'
assert 100 'struct S { a: i32 } fn f(s: S) -> S { s } fn main() -> i32 { f(S { a: 100 }).a }'
assert 5 'struct S { a: i32, b: i32 } fn f() -> S { S { a: 3, b: 5 } } fn main() -> i32 { f().b }'
assert 7 'struct S { a: i32 } fn f(b: i32, c: i32) -> S { S { a: b + c } } fn main() -> i32 { f(3, 4).a }'
