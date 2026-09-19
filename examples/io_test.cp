include "std/io.cp";

fn main() -> i32 {
    let ok1: bool = write_whole_file("demo_out.txt", "hello from C-Prime\n", 19);
    println("wrote file:");
    println(ok1);

    let mut data: *mut char = null;
    let mut len: usize = 0;
    let ok2: bool = read_whole_file("demo_out.txt", &mut data, &mut len);
    println("read file ok:");
    println(ok2);
    println("read bytes:");
    println(len);
    println("contents:");
    println(data);
    free(data as *mut void);
    return 0;
}