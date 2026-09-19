include "std/vec.cp";
include "std/map.cp";
include "std/string.cp";
include "std/io.cp";

extern fn printf(fmt: *mut char, ...) -> i32;

fn is_space(c: char) -> bool {
    let sp: char = ' ' as char;
    if c == sp {
        return true;
    }
    let tab: char = '\t' as char;
    if c == tab {
        return true;
    }
    let nl: char = '\n' as char;
    if c == nl {
        return true;
    }
    let cr: char = '\r' as char;
    return c == cr;
}



fn next_word(data: *mut char, size: usize, offset: *mut usize, word: *mut String) -> bool {
    let mut i: usize = offset[0];
    while i < size {
        let c: char = data[i];
        if is_space(c) {
            i = i + 1;
        } else {
            break;
        }
    }
    if i >= size {
        return false;
    }
    word[0].clear();
    while i < size {
        let c: char = data[i];
        if is_space(c) {
            break;
        }
        word[0].push(c);
        i = i + 1;
    }
    offset[0] = i;
    return true;
}

fn main() -> i32 {
    let mut data: *mut char = null;
    let mut size: usize = 0;
    let ok: bool = read_whole_file("wfreq_input.txt", &mut data, &mut size);
    if !ok {
        println("cannot read wfreq_input.txt");
        return 1;
    }

    let mut counts: StrIntMap = StrIntMap::new();
    let mut word: String = String::new();
    let mut offset: usize = 0;
    while next_word(data, size, &mut offset, &mut word) {
        counts.increment(word.cstr());
    }

    println("word counts:");
    let mut i: usize = 0;
    while i < counts.len() {
        let w = counts.key(i);
        let c = counts.count(i);
        printf("%s %lld\n", w, c);
        i = i + 1;
    }
    println("distinct words:");
    println(counts.len());

    counts.free_buf();
    word.free_buf();
    free(data as *mut void);
    return 0;
}