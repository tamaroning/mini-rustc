extern "C" {
    fn printf(s: &str) -> i32;
}

fn main() -> () {
    unsafe {
        printf("Hello world!\n");
    };
}
