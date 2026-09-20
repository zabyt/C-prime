include "std/vec.cp";
include "std/hashmap.cp";

extern fn printf(fmt: *const char, ...) -> i32;

fn main() -> i32 {
    let mut scores: HashMapStr[i64] = HashMapStr::new();
    scores.put("alice", 95);
    scores.put("bob", 87);
    scores.put("carol", 91);
    scores.put("dave", 78);
    scores.put("eve", 88);

    printf("entries: %lld\n", scores.len() as i64);
    if scores.contains("dave") {
        printf("dave scored %lld\n", scores.find("dave").value);
    }
    scores.remove("dave");
    if !scores.contains("dave") {
        printf("dave removed\n");
    }
    scores.put("frank", 84);
    scores.put("grace", 92);

    let names = scores.keys();
    let mut i: usize = 0;
    while i < names.len() {
        printf("  %s: %lld\n", names.get(i), scores.find(names.get(i) as *const char).value);
        i = i + 1;
    }
    names.free_buf();

    let mut counts: HashMapI64[i64] = HashMapI64::new();
    let mut n: i64 = 0;
    while n < 100 {
        counts.put(n * 13, n * n);
        n = n + 1;
    }
    if counts.find(13 * 77).value != 77 * 77 { return 1; }
    if !counts.contains(13 * 50) { return 2; }
    printf("100 keys in HashMapI64, total entries: %lld\n", counts.len() as i64);

    let mut seen: HashSetStr = HashSetStr::new();
    seen.add("apple");
    seen.add("banana");
    seen.add("apple");
    seen.add("cherry");
    printf("unique fruits: %lld\n", seen.len() as i64);
    if !seen.contains("banana") { return 3; }
    seen.remove("banana");
    if seen.contains("banana") { return 4; }
    printf("banana removed\n");

    scores.free_buf();
    counts.free_buf();
    seen.free_buf();
    return 0;
}