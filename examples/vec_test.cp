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

    let mut w: Vec[i32] = Vec::new();
    w.push(10);
    w.push(20);
    w.push(30);
    println("insert 99 at 1:");
    w.insert(1, 99);
    println(w.len() == 4);
    println(w.get(1) == 99);
    println(w.get(0) == 10);
    println(w.get(2) == 20);
    println("last:");
    println(w.last() == 30);
    println("remove at 2:");
    let removed: i32 = w.remove(2);
    println(removed == 20);
    println("len after remove:");
    println(w.len() == 3);
    println(w.get(2) == 30);
    println("last after remove:");
    println(w.last() == 30);
    w.free_buf();

    return 0;
}

