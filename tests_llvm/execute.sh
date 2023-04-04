#!/bin/bash
cd $(dirname $0)
RUSTC="../target/debug/mini-rustc"
TMP="../tmp.ll"
EXE="../tmp"
LLC="llc"

RED='\033[0;31m'
GREEN='\033[0;32m'
GRAY='\033[0;30m'
NC='\033[0m' # No Color

assert() {
    expected="$1"
    input="$2"

    rm $TMP $EXE
    $RUSTC "$input" --llvm >$TMP
    $LLC -o $EXE $TMP
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
assert 0 'fn main() -> i32 { 0 }'
