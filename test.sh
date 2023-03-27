#!/bin/bash
RUSTC="./target/debug/mini-rustc"
TMP="./tmp.s"
EXE="./tmp"
CC="cc"

assert() {
  expected="$1"
  input="$2"

  rm $TMP $EXE
  $RUSTC "$input" > $TMP
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
  $RUSTC "$input" >& /dev/null
  code="$?"
  if [ "$code" = 1 ]; then
    echo "$input => Failed to compile"
  else
    echo "$input => Successfully compiled (Unexpected)"
    exit 1
  fi
}

cargo build

assert 42 'fn main() { return 42; }'
assert 6 'fn main() { return 1+2+3; }'
assert 80 'fn main() { return 20*4; }'
assert 5 'fn main() { return 2*5+4-3*3; }'
assert 150 'fn main() { return 10*(4+5+6); }'
assert 13 'fn main() { return (((1))+(((4*((((3)))))))); }'
assert 5 'fn main() { return +3+2; }'
assert 4 'fn main() { return -3+7; }'
assert 11 'fn main() { return 3+8; return 4+6; }'
assert 0 'fn main() { let a: i32; let b: i32; return 0; }'
assert 128 'fn main() { let a: i32; a=120; a=a+8; return a; }'
assert 1 'fn main() { let a: i32; let b: i32; a=1; b=100; return a; }'
assert 1 'fn main() { return true; }'

compile_fail 'fn main() { a; return 0; }'
compile_fail 'fn main() { let a: i32; let b: i32; a=10; a=(a=10); return a; }'
compile_fail 'fn main() { let b: bool; b = 100; }'
compile_fail 'fn main() { let u: (); u = (return 0); }'

echo OK
