include "std/vec.cp";
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

    s.clear();
    s.append("C-Prime Compiler");
    println("starts_with C-Prime:");
    println(s.starts_with("C-Prime"));
    println("ends_with Compiler:");
    println(s.ends_with("Compiler"));
    println("index_of P (expect 2):");
    println(s.index_of('P' as char) == 2);
    println("index_of x (expect -1):");
    println(s.index_of('x' as char) == -1);
    println("contains r:");
    println(s.contains('r' as char));
    let lc: String = s.to_lower();
    println("lower:");
    println(lc.cstr());
    let uc: String = s.to_upper();
    println("upper:");
    println(uc.cstr());
    println("substring 8,8:");
    let sub2: String = s.substring(8, 8);
    println(sub2.cstr());
    println("compare same:");
    let cmp2: i32 = s.compare("C-Prime Compiler");
    println(cmp2 == 0);
    let ws: String = String::from_cstr("   spaced out   ");
    let tr: String = ws.trimmed();
    println("trimmed:");
    println(tr.cstr());
    println("trimmed len (expect 10):");
    println(tr.len() == 10);
    let csv: String = String::from_cstr("a;bb;ccc;");
    println("split count (expect 4):");
    let parts: Vec[String] = csv.split(';' as char);
    println(parts.len() == 4);
    println("parts[1]:");
    let p1: String = parts.get(1);
    println(p1.cstr());
    let mut k: usize = 0;
    while k < parts.len() {
        let part: String = parts.get(k);
        part.free_buf();
        k = k + 1;
    }
    parts.free_buf();
    sub2.free_buf();
    uc.free_buf();
    lc.free_buf();
    tr.free_buf();
    ws.free_buf();

    s.free_buf();
    return 0;
}