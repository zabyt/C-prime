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

fn count_words(data: *const char) -> i64 {
    let mut i: i64 = 0;
    let mut words: i64 = 0;
    let mut in_word: bool = false;
    while data[i as usize] != 0 as char {
        let c = data[i as usize];
        if is_space(c) {
            in_word = false;
        } else {
            if !in_word {
                words = words + 1;
            }
            in_word = true;
        }
        i = i + 1;
    }
    return words;
}

fn main() -> i32 {
    let args = split_args(cmd_line());
    let mut path: String = String::from_cstr("wc_input.txt");
    if args.len() >= 2 as usize {
        path = args.get(1);
    }
    let data: *mut char = read_file_all(path.cstr());
    if data == null {
        println("wc: cannot open file");
        return 1;
    }
    let mut lines: i64 = 0;
    let mut bytes: i64 = 0;
    while data[bytes as usize] != 0 as char {
        if data[bytes as usize] == '\n' as char {
            lines = lines + 1;
        }
        bytes = bytes + 1;
    }
    let words = count_words(data);
    println("{} {} {} {}", lines, words, bytes, path.cstr());
    return 0;
}