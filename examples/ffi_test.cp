import std.io;

extern fn malloc(size: usize) -> *mut void;
extern fn free(p: *mut void);
extern fn strlen(s: *const char) -> usize;
extern fn toupper(c: char) -> char;

fn main() -> i32 {
    let s: *const char = "hello ffi";
    println("length:");
    println(strlen(s));
    println("uppercase:");
    println(toupper('a'));

    let buf: *mut void = malloc(64);
    if buf == null {
        println("malloc failed");
        return 1;
    }
    println("malloc ok");
    free(buf);
    return 0;
}
