include "../std/sys.cp";
include "../std/string.cp";
include "../std/vec.cp";

extern fn system(cmd: *const char) -> i32;

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
    let mut path: String = String::from_cstr(".");
    if args.len() >= 2 as usize {
        path = args.get(1);
    }
    let mut cmd: String = String::from_cstr("dir /b ");
    let mut i: i64 = 0;
    while path.cstr()[i as usize] != 0 as char {
        cmd.push(path.cstr()[i as usize]);
        i = i + 1;
    }
    let res: i32 = system(cmd.cstr());
    if res != 0 {
        return 1;
    }
    return 0;
}