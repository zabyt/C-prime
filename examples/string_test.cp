include "std/string.cp";

fn main() -> i32 {
    let mut s: String = String::new();
    s.push('H' as char);
    s.push('i' as char);
    s.append(" from ");
    s.append("C-Prime");
    println("s:");
    println(s.cstr());

    let mut t: String = String::from_i64(123456789 as i64);
    t.set(1, 'x');
    println("t:");
    println(t.cstr());

    let mut n: String = String::from_i32(-42);
    println("n:");
    println(n.cstr());

    println("len exact:");
    println(s.len() == 15);
    println("empty:");
    println(s.is_empty());

    let mut u: String = String::with_capacity(64);
    println("capacity:");
    println(u.capacity());

    let c: char = s.at(3);
    println("char at 3 is f:");
    println(c == 'f' as char);

    u.free_buf();
    t.free_buf();
    n.free_buf();

    let pi: String = String::from_f64(3.14159);
    println("pi:");
    println(pi.cstr());
    println("pi has 8 chars:");
    println(pi.len() == 8);
    pi.free_buf();

    let pi2: String = String::from_f64(123.456);
    println("pi2:");
    println(pi2.cstr());
    let parsed: i64 = pi2.parse_i64();
    println("parsed 123:");
    println(parsed == 123);
    pi2.free_buf();

    s.free_buf();
    return 0;
}