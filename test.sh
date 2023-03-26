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
  rm $TMP $EXE
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

assert 0 "0;"
assert 42 "42;"
assert 6 "1+2+3;"
assert 80 "20*4;"
assert 5 "2*5+4-3*3;"
assert 150 "10*(4+5+6);"
assert 13 "(((1))+(((4*((((3))))))));"
assert 5 "+3+2;"
assert 4 "-3+7;"
assert 10 "3+8;4+6;"
assert 0 "let a; let b;"
assert 128 "let a; a=120; a=a+8; a;"
assert 1 "let a; let b; a=1; b=100; a;"

compile_fail "a;"
compile_fail "let a; let b; a=10; a=(a=10);"
