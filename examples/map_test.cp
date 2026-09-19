include "std/vec.cp";
include "std/map.cp";

fn main() -> i32 {
    let mut m: IntMap = IntMap::new();
    m.set(1, 10);
    m.set(2, 20);
    m.set(3, 30);
    println("len:");
    println(m.len());

    println("get 2:");
    println(m.get(2));

    println("contains 3:");
    println(m.contains(3));
    println("contains 9:");
    println(m.contains(9));

    m.set(2, 99);
    println("get 2 after update:");
    println(m.get(2));

    m.remove(1);
    println("len after remove:");
    println(m.len());
    println("contains 1:");
    println(m.contains(1));
    println("get missing returns 0:");
    println(m.get(1));

    m.free_buf();
    return 0;
}