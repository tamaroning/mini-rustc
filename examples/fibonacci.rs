fn fib(n: i32) -> i32 {
    let f: i32 = if n == 0 {
        1
    } else if n == 1 {
        1
    } else {
        fib(n - 1) + fib(n - 2)
    };
    f
}

fn main() -> i32 {
    // Run `echo $?` after executing this binary to see the result
    fib(10)
}
