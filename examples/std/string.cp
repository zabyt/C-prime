








include "vec.cp";
struct String {
    data: *mut char,
    length: usize,
    capacity: usize,
}

extern fn malloc(size: usize) -> *mut void;
extern fn free(ptr: *mut void) -> void;
extern fn realloc(ptr: *mut void, size: usize) -> *mut void;
extern fn strlen(s: *const char) -> usize;
extern fn strcmp(a: *const char, b: *const char) -> i32;
extern fn snprintf(buffer: *mut char, size: usize, format: *mut char, ...) -> i32;
extern fn atoll(p: *const char) -> i64;
extern fn toupper(c: char) -> char;
extern fn tolower(c: char) -> char;

fn is_space_char(c: char) -> bool {
    if c == ' ' as char {
        return true;
    }
    if c == '\t' as char {
        return true;
    }
    if c == '\n' as char {
        return true;
    }
    if c == '\r' as char {
        return true;
    }
    return false;
}

impl String {
    fn new() -> String {
        let mut s: String = String { data: null, length: 0, capacity: 0 };
        s.grow(16);
        s.data[0] = 0 as char;
        return s;
    }

    fn with_capacity(cap: usize) -> String {
        let mut s: String = String::new();
        s.reserve(cap);
        return s;
    }

    fn from_cstr(s: *const char) -> String {
        let mut out: String = String::new();
        out.append(s);
        return out;
    }

    fn len(&self) -> usize {
        return self.length;
    }

    fn capacity(&self) -> usize {
        return self.capacity;
    }

    fn is_empty(&self) -> bool {
        return self.length == 0;
    }

    fn at(&self, i: usize) -> char {
        return self.data[i];
    }

    fn set(&mut self, i: usize, c: char) {
        self.data[i] = c;
    }

    fn cstr(&self) -> *const char {
        return self.data as *const char;
    }

    fn reserve(&mut self, extra: usize) {
        let required = self.length + extra + 1;
        if required > self.capacity {
            let mut new_cap = required;
            if self.capacity * 2 > new_cap {
                new_cap = self.capacity * 2;
            }
            self.grow(new_cap);
        }
    }

    fn grow(&mut self, new_cap: usize) {
        if self.data == null {
            self.data = malloc(new_cap) as *mut char;
        } else {
            self.data = realloc(self.data as *mut void, new_cap) as *mut char;
        }
        self.capacity = new_cap;
    }

    fn push(&mut self, c: char) {
        if self.length + 2 > self.capacity {
            self.grow(self.capacity * 2 + 8);
        }
        self.data[self.length] = c;
        self.length = self.length + 1;
        self.data[self.length] = 0 as char;
    }

    fn append(&mut self, s: *const char) {
        let n = strlen(s);
        let mut i: usize = 0;
        while i < n {
            self.push(s[i]);
            i = i + 1;
        }
    }

    fn equals_cstr(&self, s: *const char) -> bool {
        return strcmp(self.cstr(), s) == 0;
    }

    fn clear(&mut self) {
        self.length = 0;
        if self.data != null {
            self.data[0] = 0 as char;
        }
    }

    fn free_buf(&self) {
        free(self.data as *mut void);
    }
}



fn write_i64(value: i64, buf: *mut char) -> usize {
    let mut i: usize = 0;
    if value < 0 {
        buf[0] = '-' as char;
        i = 1;
    }
    let mut v: i64 = value;
    if v < 0 {
        v = 0 - v;
    }
    let mut digits: [char; 20] = [0 as char; 20];
    let mut n: usize = 0;
    if v == 0 {
        digits[0] = '0' as char;
        n = 1;
    }
    while v > 0 {
        digits[n] = (48 + v % 10) as char;
        n = n + 1;
        v = v / 10;
    }
    let mut k: usize = n;
    while k > 0 {
        k = k - 1;
        buf[i] = digits[k];
        i = i + 1;
    }
    return i;
}

impl String {
    fn from_i64(value: i64) -> String {
        let mut s: String = String::new();
        let mut buf: [char; 24] = [0 as char; 24];
        let n = write_i64(value, buf);
        let mut j: usize = 0;
        while j < n {
            s.push(buf[j]);
            j = j + 1;
        }
        return s;
    }

    fn from_i32(value: i32) -> String {
        return String::from_i64(value as i64);
    }

    fn from_f64(value: f64) -> String {
        
        
        let mut buf: [char; 400] = [0 as char; 400];
        let n = (snprintf(buf, 400, "%f", value)) as usize;
        let mut s: String = String::new();
        let mut j: usize = 0;
        while j < n {
            s.push(buf[j]);
            j = j + 1;
        }
        return s;
    }

    fn parse_i64(&self) -> i64 {
        return atoll(self.cstr());
    }

    fn index_of(&self, c: char) -> i64 {
        let mut i: usize = 0;
        while i < self.length {
            if self.data[i] == c {
                return i as i64;
            }
            i = i + 1;
        }
        return -1;
    }

    fn contains(&self, c: char) -> bool {
        return self.index_of(c) >= 0;
    }

    fn starts_with(&self, p: *const char) -> bool {
        let n = strlen(p);
        if n > self.length {
            return false;
        }
        let mut i: usize = 0;
        while i < n {
            if p[i] != self.data[i] {
                return false;
            }
            i = i + 1;
        }
        return true;
    }

    fn ends_with(&self, p: *const char) -> bool {
        let n = strlen(p);
        if n > self.length {
            return false;
        }
        let off = self.length - n;
        let mut i: usize = 0;
        while i < n {
            if p[i] != self.data[off + i] {
                return false;
            }
            i = i + 1;
        }
        return true;
    }

    fn compare(&self, other: *const char) -> i32 {
        return strcmp(self.cstr(), other);
    }

    fn substring(&self, start: usize, count: usize) -> String {
        let mut s: String = String::new();
        let mut i: usize = start;
        while i < start + count && i < self.length {
            s.push(self.data[i]);
            i = i + 1;
        }
        return s;
    }

    fn trimmed(&self) -> String {
        let mut s: String = String::new();
        let mut lo: usize = 0;
        while lo < self.length && is_space_char(self.data[lo]) {
            lo = lo + 1;
        }
        let mut hi: usize = self.length;
        while hi > lo && is_space_char(self.data[hi - 1]) {
            hi = hi - 1;
        }
        let mut i: usize = lo;
        while i < hi {
            s.push(self.data[i]);
            i = i + 1;
        }
        return s;
    }

    fn to_upper(&self) -> String {
        let mut s: String = String::new();
        let mut i: usize = 0;
        while i < self.length {
            s.push(toupper(self.data[i]));
            i = i + 1;
        }
        return s;
    }

    fn to_lower(&self) -> String {
        let mut s: String = String::new();
        let mut i: usize = 0;
        while i < self.length {
            s.push(tolower(self.data[i]));
            i = i + 1;
        }
        return s;
    }

    fn take_buf(&mut self) -> *mut char {
        let p = self.data;
        self.data = null;
        self.length = 0;
        self.capacity = 0;
        return p;
    }

    fn split(&self, d: char) -> Vec[String] {
        let mut out: Vec[String] = Vec::new();
        let mut start: usize = 0;
        let mut i: usize = 0;
        while i <= self.length {
            let is_split = i == self.length || self.data[i] == d;
            if is_split {
                let part: String = self.substring(start, i - start);
                out.push(part);
                start = i + 1;
            }
            i = i + 1;
        }
        return out;
    }
}