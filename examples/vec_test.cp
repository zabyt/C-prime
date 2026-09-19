include "std/vec.cp";

fn main() -> i32 {
    let mut v: Vec[i32] = Vec::new();
    let mut i: i32 = 0;
    while i < 100 {
        v.push(i * 10);
        i = i + 1;
    }
    println("len:");
    println(v.len());
    println("cap:");
    println(v.capacity());
    println("sum:");
    let mut sum: i64 = 0;
    let mut j: usize = 0;
    while j < v.len() {
        let g: i32 = v.get(j);
        sum = sum + g as i64;
        j = j + 1;
    }
    println(sum);
println("popped:");
    println(v.pop());
    v.free_buf();
    
    return 0;
}

