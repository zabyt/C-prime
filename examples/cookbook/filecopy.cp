include "../std/sys.cp";
include "../std/string.cp";
include "../std/vec.cp";

fn is_space(c: char) -> bool {
    return c == ' ' as char || c == '\t' as char || c == '\n' as char || c == '\r' as char;
}

fn split_args(line: *const char) -> Vec[String] {
    let mut out: Vec[String] = Vec::new();
    let mut cur: String = String::new();
    let mut i: i64 = 0;
    while line[i as usize] != 0 as char {
        let c = line[i as usize];
        if is_space(c) {
            if cur.len() > 0 {
                out.push(cur);
                cur = String::new();
            }
        } else {
            cur.push(c);
        }
        i = i + 1;
    }
    if cur.len() > 0 {
        out.push(cur);
    }
    return out;
}

fn main() -> i32 {
    let args = split_args(cmd_line());
    let mut src: String = String::from_cstr("input.txt");
    let mut dst: String = String::from_cstr("output.txt");
    if args.len() >= 3 as usize {
        src = args.get(1);
        dst = args.get(2);
    }
    let data: *mut char = read_file_all(src.cstr());
    if data == null {
        println("filecopy: cannot read source");
        return 1;
    }
    let mut n: usize = 0;
    while data[n] != 0 as char {
        n = n + 1;
    }
    if write_file_all(dst.cstr(), data, n) != 0 {
        println("filecopy: cannot write destination");
        return 1;
    }
    println("copied {} bytes from {} to {}", n, src.cstr(), dst.cstr());
    return 0;
}