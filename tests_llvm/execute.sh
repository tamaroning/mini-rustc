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
    $RUSTC "$input" --llvm > $TMP
    $LLC -o $ASM $TMP
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

cd ..
cargo build
cd tests_llvm

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
