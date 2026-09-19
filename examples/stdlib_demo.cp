include "std/vec.cp";
include "std/string.cp";
include "std/io.cp";

fn main() -> i32 {
    let mut nums: Vec[i64] = Vec::new();
    let mut k: i32 = 0;
    while k < 10 {
        nums.push((k as i64) * (k as i64));
        k = k + 1;
    }
    println("num elements:");
    println(nums.len());

    let mut sum: i64 = 0;
    let mut j: usize = 0;
    while j < nums.len() {
        sum = sum + nums.get(j);
        j = j + 1;
    }
    println("sum of vec:");
    println(sum);
    nums.free_buf();

    let mut line: String = String::new();
    line.append("The sum is ");
    let mut part: String = String::from_i64(sum);
    line.append(part.cstr());
    line.push('\n' as char);
    println("string built:");
    println(line.cstr());

    let mut pi: String = String::from_f64(3.14159);
    println("pi formatted:");
    println(pi.cstr());
    let parsed: i64 = pi.parse_i64();
    println("pi parsed as i64:");
    println(parsed);
    pi.free_buf();

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

    let written: bool = write_whole_file("demo_out.txt", line.cstr(), line.len());
    println("wrote string ok:");
    println(written);

    let mut data2: *mut char = null;
    let mut len2: usize = 0;
    let ok3: bool = read_whole_file("demo_out.txt", &mut data2, &mut len2);
    println("read back bytes:");
    println(len2);
    println("read back contents:");
    println(data2);
    free(data2 as *mut void);

    line.free_buf();
    part.free_buf();
    return 0;
}