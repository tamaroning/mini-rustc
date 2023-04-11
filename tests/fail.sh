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

compile_fail() {
  input="$1"
  $RUSTC "$input" #>&/dev/null
  code="$?"
  if [ "$code" = 1 ]; then
    echo -e "[${GREEN}OK${NC}] $input"
  else
    echo -e "[${RED}ERROR${NC}] $input ${GRAY}=> Unexpectedly exit with code $code${NC}"
    exit 1
  fi
}

QT="'"

echo "===== Failure Tests ====="
# undeclared var
compile_fail 'fn main() -> i32 { a; return 0; }'
# empty func body returns unit
compile_fail 'fn main() -> i32 { }'
# assign number to bool
compile_fail 'fn main() -> i32 { let b: bool; b = 100; }'
# assign ! to ()
compile_fail 'fn main() -> i32 { let u: (); u = (return 0); }'
# ill-typed arithmetic
compile_fail 'fn main() -> i32 { return (1+true)*2; }'
# unexpected type of return value
compile_fail 'fn main() -> i32 { return true; }'
# unexpected type of block expression
compile_fail 'fn main() -> i32 { let a: i32; a = { 1; true }; }'
# mismatch number of arguments
compile_fail 'fn take_three(a: i32, b: i32, c: i32) -> () { } fn main() -> i32 { take_three(1, 2); 0 }'
# mismatch type of argument
compile_fail 'fn take_bool(b: bool) -> () { } fn main() -> i32 { take_bool(0); 0 }'
# type of let statement
compile_fail 'fn main() -> i32 { { let unit: () = (); } }'
# scope
compile_fail 'fn main() -> () { { let a: () = (); } a }'
# array expr with no element
compile_fail 'fn main() -> () { []; }'
compile_fail 'fn main() -> () { let a: [i32; 1] = [1, 2]; }'
compile_fail 'fn main() -> () { let a: [i32; 1] = [true]; }'
compile_fail 'fn main() -> () { let a: [i32; 1]; a[0] = true; }'
# if
compile_fail 'fn main() -> () { if (true) { } else { 1 } }'
# name space
compile_fail 'mod a { fn f() -> () { } } fn main() -> () { f() }'
# name space
compile_fail 'mod a mod b { { fn f() -> () { } } } fn main() -> () { f() }'

