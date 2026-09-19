








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
}