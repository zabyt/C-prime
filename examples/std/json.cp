extern fn malloc(size: usize) -> *mut void;
extern fn free(ptr: *mut void) -> void;
extern fn realloc(ptr: *mut void, size: usize) -> *mut void;
extern fn strlen(s: *const char) -> usize;
extern fn strcmp(a: *const char, b: *const char) -> i32;
extern fn atof(s: *const char) -> f64;

struct JsonNode {
    kind: i32,
    bval: bool,
    nval: f64,
    sval: *mut char,
    key: *mut char,
    first_child: i64,
    next_sibling: i64,
}

struct JsonParser {
    nodes_data: *mut JsonNode,
    nodes_len: usize,
    nodes_cap: usize,
    root_idx: i64,
    src: *mut char,
    len: usize,
    pos: usize,
    ok: bool,
}

impl JsonParser {
    fn add_node(&mut self, kind: i32, bval: bool, nval: f64, sval: *mut char, key: *mut char) -> i64 {
        if self.nodes_len == self.nodes_cap {
            self.nodes_cap = self.nodes_cap * 2 + 4;
            self.nodes_data = realloc(self.nodes_data as *mut void, sizeOf[JsonNode] * self.nodes_cap) as *mut JsonNode;
        }
        let mut node: JsonNode = JsonNode { kind: kind, bval: bval, nval: nval, sval: sval, key: key, first_child: -1, next_sibling: -1 };
        self.nodes_data[self.nodes_len] = node;
        let idx: i64 = self.nodes_len as i64;
        self.nodes_len = self.nodes_len + 1;
        return idx;
    }

    fn link_child(&mut self, parent: i64, child: i64) {
        let old: JsonNode = self.nodes_data[parent as usize];
        let mut head: i64 = old.first_child;
        if head < 0 {
            let node: JsonNode = JsonNode { kind: old.kind, bval: old.bval, nval: old.nval, sval: old.sval, key: old.key, first_child: child, next_sibling: old.next_sibling };
            self.nodes_data[parent as usize] = node;
            return;
        }
        while true {
            let cur: JsonNode = self.nodes_data[head as usize];
            let next: i64 = cur.next_sibling;
            if next < 0 {
                let node: JsonNode = JsonNode { kind: cur.kind, bval: cur.bval, nval: cur.nval, sval: cur.sval, key: cur.key, first_child: cur.first_child, next_sibling: child };
                self.nodes_data[head as usize] = node;
                return;
            }
            head = next;
        }
    }

    fn set_child_key(&mut self, id: i64, key: *mut char) {
        let old: JsonNode = self.nodes_data[id as usize];
        let node: JsonNode = JsonNode { kind: old.kind, bval: old.bval, nval: old.nval, sval: old.sval, key: key, first_child: old.first_child, next_sibling: old.next_sibling };
        self.nodes_data[id as usize] = node;
    }

    fn at_end(&mut self) -> bool {
        return self.pos >= self.len;
    }

    fn peek(&mut self) -> char {
        if self.pos < self.len {
            return self.src[self.pos];
        }
        return 0 as char;
    }

    fn bump(&mut self) {
        self.pos = self.pos + 1;
    }

    fn eat(&mut self, c: char) -> bool {
        if !self.at_end() && self.src[self.pos] == c {
            self.pos = self.pos + 1;
            return true;
        }
        return false;
    }

    fn skip_ws(&mut self) {
        while true {
            if self.at_end() {
                return;
            }
            let c = self.peek();
            if c == ' ' as char || c == '\t' as char || c == '\n' as char || c == '\r' as char {
                self.bump();
            } else {
                return;
            }
        }
    }

    fn parse_string(&mut self) -> *mut char {
        if !self.eat('"' as char) {
            self.ok = false;
            return null;
        }
        let mut s: String = String::new();
        while !self.at_end() {
            let c = self.peek();
            if c == '"' as char {
                self.bump();
                return s.take_buf();
            }
            if c == '\\' as char {
                self.bump();
                if self.at_end() {
                    self.ok = false;
                    break;
                }
                let e = self.peek();
                if e == '"' as char || e == '\\' as char || e == '/' as char {
                    s.push(e);
                    self.bump();
                } else if e == 'b' as char {
                    s.push('\b' as char);
                    self.bump();
                } else if e == 'f' as char {
                    s.push('\f' as char);
                    self.bump();
                } else if e == 'n' as char {
                    s.push('\n' as char);
                    self.bump();
                } else if e == 'r' as char {
                    s.push('\r' as char);
                    self.bump();
                } else if e == 't' as char {
                    s.push('\t' as char);
                    self.bump();
                } else if e == 'u' as char {
                    self.bump();
                    let mut k: usize = 0;
                    while k < 4 {
                        if self.at_end() {
                            self.ok = false;
                            break;
                        }
                        self.bump();
                        k = k + 1;
                    }
                } else {
                    self.ok = false;
                    break;
                }
            } else {
                s.push(c);
                self.bump();
            }
        }
        let out: *mut char = s.take_buf();
        self.ok = false;
        return out;
    }

    fn parse_number(&mut self) -> f64 {
        let mut buf: [char; 64] = [0 as char; 64];
        let mut n: usize = 0;
        while !self.at_end() {
            let ch = self.peek();
            let dv: i64 = ch as i64;
            let is_digit = dv >= 48 && dv <= 57;
            if is_digit || ch == '-' as char || ch == '+' as char || ch == '.' as char || ch == 'e' as char || ch == 'E' as char {
                if n < 63 {
                    buf[n] = ch;
                    n = n + 1;
                }
                self.bump();
            } else {
                break;
            }
        }
        buf[n] = 0 as char;
        return atof(buf);
    }

    fn parse_value(&mut self) -> i64 {
        self.skip_ws();
        if self.at_end() {
            self.ok = false;
            return self.add_node(0, false, 0.0, null, null);
        }
        let c = self.peek();
        if c == '{' as char {
            return self.parse_object();
        }
        if c == '[' as char {
            return self.parse_array();
        }
        if c == '"' as char {
            let s: *mut char = self.parse_string();
            if self.ok {
                return self.add_node(3, false, 0.0, s, null);
            }
            return self.add_node(0, false, 0.0, null, null);
        }
        if c == 't' as char {
            self.bump();
            let t1 = self.eat('r' as char);
            let t2 = self.eat('u' as char);
            let t3 = self.eat('e' as char);
            if t1 && t2 && t3 {
                return self.add_node(1, true, 0.0, null, null);
            }
            self.ok = false;
            return self.add_node(0, false, 0.0, null, null);
        }
        if c == 'f' as char {
            self.bump();
            let f1 = self.eat('a' as char);
            let f2 = self.eat('l' as char);
            let f3 = self.eat('s' as char);
            let f4 = self.eat('e' as char);
            if f1 && f2 && f3 && f4 {
                return self.add_node(1, false, 0.0, null, null);
            }
            self.ok = false;
            return self.add_node(0, false, 0.0, null, null);
        }
        if c == 'n' as char {
            self.bump();
            let n1 = self.eat('u' as char);
            let n2 = self.eat('l' as char);
            let n3 = self.eat('l' as char);
            if n1 && n2 && n3 {
                return self.add_node(0, false, 0.0, null, null);
            }
            self.ok = false;
            return self.add_node(0, false, 0.0, null, null);
        }
        let num: f64 = self.parse_number();
        return self.add_node(2, false, num, null, null);
    }

    fn parse_array(&mut self) -> i64 {
        self.bump();
        self.skip_ws();
        let idx: i64 = self.add_node(4, false, 0.0, null, null);
        if self.eat(']' as char) {
            return idx;
        }
        while true {
            if self.at_end() {
                self.ok = false;
                break;
            }
            let v: i64 = self.parse_value();
            self.link_child(idx, v);
            self.skip_ws();
            if !self.eat(',' as char) {
                break;
            }
        }
        self.skip_ws();
        if !self.eat(']' as char) {
            self.ok = false;
        }
        return idx;
    }

    fn parse_object(&mut self) -> i64 {
        self.bump();
        self.skip_ws();
        let idx: i64 = self.add_node(5, false, 0.0, null, null);
        if self.eat('}' as char) {
            return idx;
        }
        while true {
            if self.at_end() {
                self.ok = false;
                break;
            }
            self.skip_ws();
            let key: *mut char = self.parse_string();
            self.skip_ws();
            if !self.eat(':' as char) {
                self.ok = false;
                free(key as *mut void);
                break;
            }
            self.skip_ws();
            let v: i64 = self.parse_value();
            self.set_child_key(v, key);
            self.link_child(idx, v);
            self.skip_ws();
            if !self.eat(',' as char) {
                break;
            }
        }
        self.skip_ws();
        if !self.eat('}' as char) {
            self.ok = false;
        }
        return idx;
    }

    fn root(&self) -> i64 {
        return self.root_idx;
    }

    fn set_root(&mut self, id: i64) {
        self.root_idx = id;
    }

    fn kind(&self, id: i64) -> i32 {
        return self.nodes_data[id as usize].kind;
    }

    fn is_null(&self, id: i64) -> bool {
        return self.kind(id) == 0;
    }

    fn as_bool(&self, id: i64) -> bool {
        return self.nodes_data[id as usize].bval;
    }

    fn as_number(&self, id: i64) -> f64 {
        return self.nodes_data[id as usize].nval;
    }

    fn as_str(&self, id: i64) -> *const char {
        return self.nodes_data[id as usize].sval as *const char;
    }

    fn member_count(&self, id: i64) -> usize {
        let mut n: usize = 0;
        let mut cur: i64 = self.nodes_data[id as usize].first_child;
        while cur >= 0 {
            n = n + 1;
            cur = self.nodes_data[cur as usize].next_sibling;
        }
        return n;
    }

    fn child_at(&self, id: i64, i: usize) -> i64 {
        let mut cur: i64 = self.nodes_data[id as usize].first_child;
        let mut k: usize = 0;
        while cur >= 0 && k < i {
            cur = self.nodes_data[cur as usize].next_sibling;
            k = k + 1;
        }
        return cur;
    }

    fn key_at(&self, id: i64, i: usize) -> *const char {
        let c: i64 = self.child_at(id, i);
        if c < 0 {
            return null;
        }
        return self.nodes_data[c as usize].key as *const char;
    }

    fn find_key(&self, id: i64, key: *const char) -> i64 {
        let mut cur: i64 = self.nodes_data[id as usize].first_child;
        while cur >= 0 {
            let k: *mut char = self.nodes_data[cur as usize].key;
            if k != null && strcmp(k, key) == 0 {
                return cur;
            }
            cur = self.nodes_data[cur as usize].next_sibling;
        }
        return -1;
    }

    fn has_key(&self, id: i64, key: *const char) -> bool {
        return self.find_key(id, key) >= 0;
    }

    fn get_key(&self, id: i64, key: *const char) -> i64 {
        return self.find_key(id, key);
    }

    fn free_buf(&self) {
        let mut i: usize = 0;
        while i < self.nodes_len {
            if self.nodes_data[i].sval != null {
                free(self.nodes_data[i].sval as *mut void);
            }
            if self.nodes_data[i].key != null {
                free(self.nodes_data[i].key as *mut void);
            }
            i = i + 1;
        }
        free(self.nodes_data as *mut void);
    }
}

fn json_parse(src: *const char, ok: *mut bool) -> JsonParser {
    let mut p: JsonParser = JsonParser { nodes_data: null, nodes_len: 0, nodes_cap: 0, root_idx: -1, src: src as *mut char, len: strlen(src), pos: 0, ok: true };
    p.skip_ws();
    p.set_root(p.parse_value());
    p.skip_ws();
    if !p.at_end() {
        p.ok = false;
    }
    ok[0] = p.ok;
    return p;
}