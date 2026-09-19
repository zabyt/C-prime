include "std/vec.cp";
include "std/string.cp";
include "std/json.cp";

fn main() -> i32 {
    let text = "{\"name\":\"C-Prime\", \"version\":1.5, \"active\":true, \"nothing\":null, \"tags\":[\"systems\",\"compiler\",42], \"meta\":{\"stars\":3, \"note\":\"beta\"}}";
    let mut ok: bool = false;
    let doc: JsonParser = json_parse(text, &mut ok);
    if !ok {
        println("json parse failed");
        return 1;
    }
    let root: i64 = doc.root();
    println("root kind (expect 5):");
    println(doc.kind(root));
    println("member count (expect 6):");
    println(doc.member_count(root));
    println("first key (expect name):");
    println(doc.key_at(root, 0));
    let name: i64 = doc.get_key(root, "name");
    println("name string:");
    println(doc.as_str(name));
    let ver: i64 = doc.get_key(root, "version");
    println("version number:");
    println(doc.as_number(ver));
    let active: i64 = doc.get_key(root, "active");
    println("active bool:");
    println(doc.as_bool(active));
    let nil: i64 = doc.get_key(root, "nothing");
    println("nothing is_null:");
    println(doc.is_null(nil));
    let tags: i64 = doc.get_key(root, "tags");
    println("tags count (expect 3):");
    println(doc.member_count(tags));
    let t0: i64 = doc.child_at(tags, 0);
    println("tags[0]:");
    println(doc.as_str(t0));
    let t2: i64 = doc.child_at(tags, 2);
    println("tags[2] number:");
    println(doc.as_number(t2));
    let meta: i64 = doc.get_key(root, "meta");
    let note: i64 = doc.get_key(meta, "note");
    println("meta note:");
    println(doc.as_str(note));
    let stars: i64 = doc.get_key(meta, "stars");
    println("meta stars:");
    println(doc.as_number(stars));
    println("has key name:");
    println(doc.has_key(root, "name"));
    println("missing key returns -1:");
    let missing: i64 = doc.get_key(root, "nope");
    println(missing < 0);
    doc.free_buf();

    let cfg: String = String::from_cstr("  the quick brown fox jumps over the lazy dog  ");
    let t: String = cfg.trimmed();
    println("trimmed:");
    println(t.cstr());
    println("starts_with the:");
    println(t.starts_with("the"));
    println("ends_with dog:");
    println(t.ends_with("dog"));
    println("contains q:");
    println(t.contains('q' as char));
    println("index_of f (expect 16):");
    println(t.index_of('f' as char));
    println("index_of z (expect 37):");
    println(t.index_of('z' as char));
    println("upper:");
    let up: String = t.to_upper();
    println(up.cstr());
    println("lower:");
    let lo: String = up.to_lower();
    println(lo.cstr());
    println("substring 10,5:");
    let sub: String = t.substring(10, 5);
    println(sub.cstr());
    println("compare equal:");
    let cmp: i32 = t.compare("the quick brown fox jumps over the lazy dog");
    println(cmp == 0);
    println("split word count (expect 9):");
    let words: Vec[String] = t.split(' ' as char);
    println(words.len());
    let mut i: usize = 0;
    while i < words.len() {
        let w: String = words.get(i);
        println(w.cstr());
        i = i + 1;
    }
    i = 0;
    while i < words.len() {
        let w: String = words.get(i);
        w.free_buf();
        i = i + 1;
    }
    words.free_buf();
    sub.free_buf();
    lo.free_buf();
    up.free_buf();
    t.free_buf();
    cfg.free_buf();
    return 0;
}