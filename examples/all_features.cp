include "std/ctypes.cp";
include "std/vec.cp";
include "std/string.cp";
include "std/map.cp";
include "std/hashmap.cp";
include "std/ini.cp";
include "std/sys.cp";
include "std/io.cp";

extern fn printf(fmt: *const char, ...) -> i32;

enum Color { Red, Green, Blue }
enum class Grade : i32 { C = 2, B = 3, A = 4 }
using Number = i32;
const ANSWER: Number = 42;

struct Point { x: f64, y: f64 }
impl Point {
    fn new(x: f64, y: f64) -> Point { return Point { x: x, y: y }; }
    fn add(self, other: Point) -> Point { return Point { x: self.x + other.x, y: self.y + other.y }; }
    fn len_sq(self) -> f64 { return self.x * self.x + self.y * self.y; }
}

struct Counter { n: i64 }
impl Counter {
    fn new() -> Counter { return Counter { n: 0 }; }
    fn bump(&mut self) { self.n = self.n + 1; }
    fn current(&self) -> i64 { return self.n; }
}

struct Box[T] { value: T }
impl Box[T] {
    fn new(v: T) -> Box[T] { return Box[T] { value: v }; }
    fn get(self) -> T { return self.value; }
}

struct Pair[A, B] { first: A, second: B }
impl Pair[A, B] {
    fn new(first: A, second: B) -> Pair[A, B] { return Pair[A, B] { first: first, second: second }; }
}

fn generic_identity[T](value: T) -> T { return value; }
fn swap_vals[T](a: T, b: T) -> T { return b; }

fn factorial(n: i32) -> i32 {
    if n <= 1 { return 1; }
    return n * factorial(n - 1);
}

fn check_i64(got: i64, want: i64, name: *const char) -> i64 {
    if got == want { printf("  ok %s\n", name); return 0; }
    printf("  FAIL %s: expected %lld got %lld\n", name, want, got);
    return 1;
}

fn check_bool(got: bool, name: *const char) -> i64 {
    if got { printf("  ok %s\n", name); return 0; }
    printf("  FAIL %s: expected true\n", name);
    return 1;
}

fn check_str(got: *const char, want: *const char, name: *const char) -> i64 {
    if strcmp(got, want) == 0 { printf("  ok %s\n", name); return 0; }
    printf("  FAIL %s: expected %s got %s\n", name, want, got);
    return 1;
}

fn main() -> i32 {
    let mut fails: i64 = 0;

    printf("== operators ==\n");
    fails = fails + check_i64(10 + 3, 13, "add");
    fails = fails + check_i64(10 - 3, 7, "sub");
    fails = fails + check_i64(10 * 3, 30, "mul");
    fails = fails + check_i64(10 / 3, 3, "div");
    fails = fails + check_i64(10 % 3, 1, "mod");
    fails = fails + check_bool(10 > 3, "gt");
    fails = fails + check_bool(3 <= 3, "le");
    fails = fails + check_bool(10 == 10, "eq");
    fails = fails + check_bool(false || true, "or");
    fails = fails + check_bool(true && !false, "and");
    fails = fails + check_i64((1 | 2) & 3, 3, "bitwise");
    fails = fails + check_i64(1 << 4, 16, "shl");
    fails = fails + check_i64(256 >> 3, 32, "shr");
    fails = fails + check_i64(5 ^ 3, 6, "xor");
    fails = fails + check_i64((~0) as i64, -1, "bitnot");

    printf("== compound assign ==\n");
    let mut m: i32 = 5;
    m += 2;
    m -= 1;
    m *= 3;
    m /= 2;
    m %= 4;
    fails = fails + check_i64(m as i64, 1, "compound");

    printf("== while/break/continue ==\n");
    let mut i: i32 = 0;
    let mut sum: i32 = 0;
    while i < 10 {
        if i == 7 { i = i + 1; continue; }
        if i == 9 { break; }
        sum = sum + i;
        i = i + 1;
    }
    fails = fails + check_i64(sum as i64, 29, "while break/cont");

    printf("== switch ==\n");
    let mut grade: i32 = 0;
    switch 82 {
        case 90: grade = 5; break;
        case 80: grade = 4; break;
        default: grade = 1; break;
    }
    fails = fails + check_i64(grade as i64, 1, "switch");

    printf("== match ==\n");
    let mut fruit: i32 = 1;
    match fruit {
        0 => printf("apple\n"),
        1 => printf("banana\n"),
        _ => printf("other\n"),
    }
    let mv: i32 = match 3 { 1 => 100, 3 => 300, _ => 0 };
    fails = fails + check_i64(mv as i64, 300, "match expr");
    match 42 {
        n => printf("binding value is %lld\n", n as i64),
    }

    printf("== enum/const/using ==\n");
    fails = fails + check_i64(Color::Green as i64, 1, "enum implicit");
    fails = fails + check_i64(Grade::A as i64, 4, "enum class");
    fails = fails + check_i64(ANSWER as i64, 42, "const alias");

    printf("== struct methods ==\n");
    let p: Point = Point::new(3.0, 4.0);
    fails = fails + check_i64(p.len_sq() as i64, 25, "method call");
    let q: Point = p.add(Point::new(1.0, 2.0));
    fails = fails + check_i64(q.x as i64, 4, "field access");
    let mut ctr: Counter = Counter::new();
    ctr.bump();
    ctr.bump();
    ctr.bump();
    fails = fails + check_i64(ctr.current(), 3, "self receiver");

    printf("== generics ==\n");
    let bv: i32 = 5;
    let b1: Box[i32] = Box::new(bv);
    fails = fails + check_i64(b1.get() as i64, 5, "generic class");
    let fi: i32 = 9;
    let sn: *const char = "nine";
    let pr: Pair[i32, *const char] = Pair::new(fi, sn);
    fails = fails + check_i64(pr.first as i64, 9, "generic pair");
    fails = fails + check_str(pr.second, "nine", "generic pair str");
    let gi: i32 = generic_identity(11);
    fails = fails + check_i64(gi as i64, 11, "generic fn");
    fails = fails + check_bool(generic_identity(true), "generic bool");
    let sw: i32 = swap_vals(1, 2);
    fails = fails + check_i64(sw as i64, 2, "generic swap");
    fails = fails + check_i64(factorial(6) as i64, 720, "recursion");

    printf("== pointers ==\n");
    let mut value: i32 = 123;
    let ptr: *mut i32 = &mut value;
    *ptr = 456;
    let rp: *const i32 = &value;
    fails = fails + check_i64((*rp) as i64, 456, "ptr deref");
    let np: *mut i32 = null;
    fails = fails + check_bool(np == null, "null");
    let heap: *mut i32 = malloc(4) as *mut i32;
    heap[0] = 77;
    fails = fails + check_i64(heap[0] as i64, 77, "malloc+index");
    free(heap as *mut void);

    printf("== casts ==\n");
    let cn: i32 = 3.9 as i32;
    fails = fails + check_i64(cn as i64, 3, "f2i");
    let cc: char = (65) as char;
    fails = fails + check_bool(cc == 'A' as char, "i2c");
    let cnum: i64 = cc as i64;
    fails = fails + check_i64(cnum, 65, "c2i");

    printf("== vec ==\n");
    let mut v: Vec[i32] = Vec::new();
    v.push(10);
    v.push(20);
    v.push(30);
    fails = fails + check_i64(v.len() as i64, 3, "vec len");
    fails = fails + check_i64(v.get(1) as i64, 20, "vec get");
    v.set(1, 99);
    fails = fails + check_i64(v.get(1) as i64, 99, "vec set");
    let popped: i32 = v.pop();
    fails = fails + check_i64(popped as i64, 30, "vec pop");
    let mut vsum: i32 = 0;
    for e in v {
        vsum = vsum + e;
    }
    fails = fails + check_i64(vsum as i64, 109, "for vec");
    v.free_buf();

    printf("== string ==\n");
    let mut s: String = String::new();
    s.push('H' as char);
    s.push('i');
    s.append(" C-Prime");
    fails = fails + check_i64(s.len() as i64, 10, "str len");
    fails = fails + check_bool(s.at(0) == 'H' as char, "str at");
    fails = fails + check_bool(s.contains('P' as char), "str contains");
    fails = fails + check_i64(s.index_of('P') as i64, 5, "str index_of");
    fails = fails + check_bool(s.starts_with("Hi"), "str starts_with");
    fails = fails + check_bool(s.ends_with("Prime"), "str ends_with");
    fails = fails + check_bool(s.equals_cstr("Hi C-Prime"), "str equals");
    let mut up: String = s.to_upper();
    fails = fails + check_bool(up.equals_cstr("HI C-PRIME"), "str to_upper");
    up.free_buf();
    let mut chcount: i64 = 0;
    for c in s {
        chcount = chcount + 1;
    }
    fails = fails + check_i64(chcount, 10, "for string");
    let numstr: String = String::from_i64(2024);
    fails = fails + check_str(numstr.cstr(), "2024", "from_i64");
    numstr.free_buf();
    let ns: String = String::from_cstr("12345");
    let parsed: i64 = ns.parse_i64();
    fails = fails + check_i64(parsed, 12345, "parse_i64");
    ns.free_buf();
    s.clear();
    fails = fails + check_bool(s.is_empty(), "str clear");
    s.free_buf();

    printf("== intmap ==\n");
    let mut im: IntMap = IntMap::new();
    im.set(1, 10);
    im.set(2, 20);
    im.set(3, 30);
    fails = fails + check_i64(im.get(2) as i64, 20, "intmap get");
    fails = fails + check_bool(im.contains(1), "intmap contains");
    im.set(2, 99);
    fails = fails + check_i64(im.get(2) as i64, 99, "intmap update");
    im.remove(1);
    fails = fails + check_bool(!im.contains(1), "intmap remove");
    im.free_buf();

    printf("== hashmap ==\n");
    let mut scores: HashMapStr[i64] = HashMapStr::new();
    scores.put("alice", 95);
    scores.put("bob", 87);
    scores.put("bob", 88);
    fails = fails + check_i64(scores.len() as i64, 2, "hash len");
    fails = fails + check_i64(scores.find("bob").value, 88, "hash val");
    fails = fails + check_bool(scores.contains("alice"), "hash contains");
    scores.remove("alice");
    fails = fails + check_bool(!scores.contains("alice"), "hash remove");
    scores.free_buf();

    printf("== hashset ==\n");
    let mut seen: HashSetStr = HashSetStr::new();
    seen.add("dup");
    seen.add("dup");
    seen.add("uniq");
    fails = fails + check_i64(seen.len() as i64, 2, "set dedup");
    fails = fails + check_bool(seen.contains("uniq"), "set contains");
    seen.free_buf();

    printf("== ini ==\n");
    let text = "[server]\nhost = 127.0.0.1\nport = 8080\n\n[logging]\nlevel = info";
    let cfg: Ini = Ini::parse(text);
    fails = fails + check_bool(cfg.valid, "ini valid");
    fails = fails + check_bool(cfg.has("server", "host"), "ini has");
    fails = fails + check_str(cfg.get("server", "host") as *const char, "127.0.0.1", "ini get");
    fails = fails + check_str(cfg.get("logging", "level") as *const char, "info", "ini get2");
    cfg.free_buf();

    printf("== sys ==\n");
    fails = fails + check_bool(time_s() > 0, "time_s");
    fails = fails + check_bool(cmd_line() != null, "cmd_line");
    fails = fails + check_bool(env_get("PATH") != null, "env_get");
    sleep_ms(1);
    let mut rng: Rng = Rng::new(time_s());
    let rr: i64 = rng.range(1, 100);
    fails = fails + check_bool(rr >= 1 && rr < 100, "rng range");

    printf("== io ==\n");
    let wok: bool = write_whole_file("allfeat_out.txt", "hello file", 10);
    fails = fails + check_bool(wok, "io write");
    let mut data: *mut char = null;
    let mut n: usize = 0;
    let rok: bool = read_whole_file("allfeat_out.txt", &mut data, &mut n);
    fails = fails + check_bool(rok, "io read");
    fails = fails + check_i64(n as i64, 10, "io read len");
    fails = fails + check_bool(strcmp(data, "hello file") == 0, "io read data");
    free(data as *mut void);

    printf("== extern C ==\n");
    fails = fails + check_i64(strlen("hello") as i64, 5, "strlen");
    fails = fails + check_i64(strcmp("abc", "abc") as i64, 0, "strcmp eq");
    fails = fails + check_i64(atoi("123") as i64, 123, "atoi");
    fails = fails + check_i64(atol("456") as i64, 456, "atol");
    fails = fails + check_bool(floor(2.9) == 2.0, "floor");
    fails = fails + check_bool(ceil(2.1) == 3.0, "ceil");
    let sq: f64 = sqrt(16.0);
    fails = fails + check_bool(sq == 4.0, "sqrt");
    let pw: f64 = pow(2.0, 8.0);
    fails = fails + check_bool(pw == 256.0, "pow");

    printf("== sizeOf ==\n");
    fails = fails + check_i64(sizeOf[i32]() as i64, 4, "sizeOf i32");
    fails = fails + check_i64(sizeOf[f64]() as i64, 8, "sizeOf f64");

    printf("\n--- all features: %lld failed ---\n", fails);
    if fails == 0 {
        printf("ALL FEATURE CHECKS PASSED\n");
        return 0;
    }
    return 1;
}
