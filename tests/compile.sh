#!/bin/bash
cd $(dirname $0)
RUSTC="../target/debug/mini-rustc"
TMP="./tmp.s"
EXE="./tmp"
CC="gcc"

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
        echo "$input => $actual"
    else
        echo "$input => $expected expected, but got $actual"
        exit 1
    fi
}

QT="'"

echo "===== Compile Tests ====="
# ref type
assert 0 'fn main() -> i32 { let string: &'$QT'static str; 0  }'
# string literal
assert 0 'fn main() -> i32 { "Hello"; "World"; 0 }'
assert 0 'fn main() -> i32 { let s: &'$QT'static str; s = "Hello, World"; 0 }'
# never type
assert 0 'fn main() -> i32 { return 0; }'
assert 0 'fn main() -> i32 { return 0 }'
assert 0 'fn main() -> () { let a: i32 = (return ()); a = return (); }'
assert 0 'fn main() -> () { { let never: ! = (return ()); } }'
assert 0 'fn main() -> () { { let unit: () = (return ()); } }'
assert 0 'fn main() -> () { let never: ! = (return ()); }'
assert 0 'fn main() -> () { let unit: () = (return ()); }'
# type of let statement
assert 0 'fn main() -> () { { let unit: () = (); } }'
assert 0 'fn main() -> () { let a: i32 = 1; }'
