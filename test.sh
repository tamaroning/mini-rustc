#!/bin/bash
RUSTC="./target/debug/mini-rustc"
TMP="./tmp.s"
EXE="./tmp"
CC="cc"

assert() {
  expected="$1"
  input="$2"

  rm $TMP $EXE
  $RUSTC "$input" >$TMP
  $CC -o $EXE $TMP
  $EXE
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

compile_fail() {
  input="$1"
  $RUSTC "$input" >&/dev/null
  code="$?"
  if [ "$code" = 1 ]; then
    echo "$input => Failed to compile"
  else
    echo "$input => Unexpectedly exit with code $code"
    exit 1
  fi
}

cargo build

assert 42 'fn main() -> i32 { return 42; }'
assert 6 'fn main() -> i32 { return 1+2+3; }'
assert 80 'fn main() -> i32 { return 20*4; }'
assert 5 'fn main() -> i32 { return 2*5+4-3*3; }'
assert 150 'fn main() -> i32 { return 10*(4+5+6); }'
assert 13 'fn main() -> i32 { return (((1))+(((4*((((3)))))))); }'
assert 5 'fn main() -> i32 { return +3+2; }'
assert 4 'fn main() -> i32 { return -3+7; }'
assert 11 'fn main() -> i32 { return 3+8; return 4+6; }'
assert 0 'fn main() -> i32 { let a: i32; let b: i32; return 0; }'
assert 128 'fn main() -> i32 { let a: i32; a=120; a=a+8; return a; }'
assert 1 'fn main() -> i32 { let a: i32; let b: i32; a=1; b=100; return a; }'
assert 5 'fn five() -> i32 { return 5; } fn main() -> i32 { return five(); }'
assert 0 'fn tru() -> bool { return true; } fn main() -> i32 { tru(); return 0; }'
assert 2 'fn main() -> i32 { return {1; 2}; }'
assert 3 'fn main() -> i32 { let a: i32; a = { 1; 2; 3 }; 4; return a; }'
assert 10 'fn main() -> i32 { 10 }'
assert 1 'fn main() -> i32 { if true { 1 } else { 0 } }'
assert 4 'fn main() -> i32 { if false { 3 } else { 4 } }'
assert 1 'fn id(n: i32) -> i32 { n } fn main() -> i32 { id(1) }'
assert 10 'fn id(n: i32) -> i32 { n } fn main() -> i32 { id(4) + id(6) }'
assert 8 'fn fib(n: i32) -> i32 { if (n == 0) { 1 } else if (n == 1) { 1 } else { fib(n-1) + fib(n-2) } } fn main() -> i32 { fib(5) }'

compile_fail 'fn main() -> i32 { a; return 0; }'
compile_fail 'fn main() -> i32 { let a: i32; let b: i32; a=10; a=(a=10); return a; }'
compile_fail 'fn main() -> i32 { let b: bool; b = 100; }'
compile_fail 'fn main() -> i32 { let u: (); u = (return 0); }'
compile_fail 'fn main() -> i32 { return (1+true)*2; }'
compile_fail 'fn main() -> i32 { return true; }'
compile_fail 'fn main() -> i32 { let a: i32; a={1; true}; }'

echo OK
