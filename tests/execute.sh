#!/bin/bash
cd $(dirname $0)
RUSTC="../target/debug/mini-rustc"
TMP="../tmp.s"
EXE="../tmp"
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
    $CC -o $EXE $TMP
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
# arithmetic
assert 42 'fn main() -> i32 { return 42; }'
assert 6 'fn main() -> i32 { return 1+2+3; }'
assert 80 'fn main() -> i32 { return 20*4; }'
assert 5 'fn main() -> i32 { return 2*5+4-3*3; }'
assert 150 'fn main() -> i32 { return 10*(4+5+6); }'
assert 13 'fn main() -> i32 { return (((1))+(((4*((((3)))))))); }'
# unary
assert 5 'fn main() -> i32 { return +3+2; }'
assert 4 'fn main() -> i32 { return -3+7; }'
# let stmt
assert 11 'fn main() -> i32 { return 3+8; return 4+6; }'
assert 0 'fn main() -> i32 { let a: i32; let b: i32; return 0; }'
assert 128 'fn main() -> i32 { let a: i32; a = 120; a = a + 8; return a; }'
assert 1 'fn main() -> i32 { let a: i32; let b: i32; a = 1; b = 100; return a; }'
# let with initalizer
assert 0 'fn main() -> i32 { let a: i32 = 0; a }'
assert 204 'fn main() -> i32 { let b: i32 = 10; let c: i32 = 20; 4 + c * b }'
# func call with no arg
assert 5 'fn five() -> i32 { return 5; } fn main() -> i32 { return five(); }'
assert 0 'fn tru() -> bool { return true; } fn main() -> i32 { tru(); return 0; }'
# block expr
assert 2 'fn main() -> i32 { return { 1; 2 }; }'
assert 3 'fn main() -> i32 { let blo: i32; blo = { 1; 2; 3 }; 4; return blo; }'
assert 10 'fn main() -> i32 { 10 }'
# if
assert 1 'fn main() -> i32 { if true { 1 } else { 0 } }'
assert 4 'fn main() -> i32 { if false { 3 } else { 4 } }'
# func call
assert 1 'fn id(n: i32) -> i32 { n } fn main() -> i32 { id(1) }'
assert 10 'fn id(n: i32) -> i32 { n } fn main() -> i32 { id(4) + id(6) }'
# recursive call
assert 8 'fn fib(n: i32) -> i32 { if n == 0 { 1 } else if n == 1 { 1 } else { fib(n-1) + fib(n-2) } } fn main() -> i32 { fib(5) }'
# array
assert 10 'fn main() -> i32 { let arr: [i32; 10]; arr[4] = 10; arr[4] }'
assert 6 'fn main() -> i32 { let arr: [i32; 5]; let arr2: [i32; 6]; arr[1 + 2] = 4; arr2[arr[3] + 1] = 6; arr2[5] }'
# empty func body
assert 0 'fn emp() -> () { } fn main() -> i32 { 0 }'
# multi-dimension array
assert 10 'fn main() -> i32 { let a: [[i32; 2]; 2]; a[1][1] = 10; a[1][1] }'
assert 10 'fn main() -> i32 { let a: [[i32; 3]; 4]; a[3][2] = 10; a[3][2] }'
# struct
assert 0 'struct Empty {} fn main() -> i32 { Empty {}; let e: Empty; e = Empty {}; 0 }'
assert 0 'struct S { n: i32, b: bool, arr: [i32; 10], } fn main() -> i32 { 0 }'
assert 0 'struct P { x: i32, y: i32, z: i32 } fn main() -> i32 { P { x: 0, y: 1, z: 2 }; 0 }'
assert 3 'struct P { x: i32, y: i32, z: i32 } fn main() -> i32 { let p: P; p = P { x: 0, y: 1, z: 2 }; p.y + p.z }'
# nested struct
assert 31 'struct Pt { x: i32, y: i32, z: i32 } struct Edge { p1: Pt, p2: Pt }
fn main() -> i32 { let e: Edge; e.p1 = Pt { x: 10, y: 20, z: 0 }; e.p2.x = 1; e.p2.y = 2; e.p1.x + e.p1.y + e.p2.x }'
# array expr
assert 3 'fn main() -> i32 { let a:[i32; 4]; a = [1, 2, 3, 4]; a[2] }'
# TODO: assert 0 'fn main() -> () { [()][0] }'