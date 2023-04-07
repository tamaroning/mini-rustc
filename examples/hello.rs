extern "C" {
    fn puts(s: &str) -> i32;
}

fn main() -> () {
    unsafe {
        puts("Hello mini-rustc!");
    };
}
