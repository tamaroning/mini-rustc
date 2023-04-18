extern "C" {
    fn puts(c: *const i32) -> i32;
}

fn main() -> () {
    unsafe {
        puts("Hello mini-rustc!" as *const str as *const i32);
    };
}
