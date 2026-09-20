include "vec.cp";

extern fn malloc(size: usize) -> *mut void;
extern fn free(ptr: *mut void) -> void;
extern fn realloc(ptr: *mut void, size: usize) -> *mut void;
extern fn memset(dst: *mut void, value: i32, size: usize) -> *mut void;
extern fn strlen(s: *const char) -> usize;
extern fn strcmp(a: *const char, b: *const char) -> i32;

struct SlotStr[V] {
    used: bool,
    deleted: bool,
    hash: usize,
    key: *mut char,
    value: V,
}

struct SlotI64[V] {
    used: bool,
    deleted: bool,
    hash: usize,
    key: i64,
    value: V,
}

fn dup_cstr(s: *const char) -> *mut char {
    let n = strlen(s);
    let out = malloc(n + 1) as *mut char;
    let mut i: usize = 0;
    while i <= n {
        out[i] = s[i];
        i = i + 1;
    }
    return out;
}

fn hash_str(s: *const char) -> usize {
    let mut h: usize = 1469598103934665603;
    let mut i: usize = 0;
    while s[i] != 0 as char {
        h = (h ^ (s[i] as usize)) * 1099511628211;
        i = i + 1;
    }
    return h;
}

fn hash_i64(x: i64) -> usize {
    let mut h: usize = (x as usize) * 2654435761;
    h = h ^ (h >> 16);
    h = h ^ (h >> 33);
    return h;
}

struct HashMapStr[V] {
    data: *mut SlotStr[V],
    size: usize,
    count: usize,
}

impl HashMapStr[V] {
    fn new() -> HashMapStr[V] {
        let mut m: HashMapStr[V] = HashMapStr[V] { data: null, size: 0, count: 0 };
        m.grow_table(8);
        return m;
    }

    fn len(&self) -> usize {
        return self.count;
    }

    fn is_empty(&self) -> bool {
        return self.count == 0;
    }

    fn grow_table(&mut self, new_size: usize) {
        let old = self.data;
        let old_size = self.size;
        let new_buf = malloc(new_size * sizeOf[SlotStr[V]]) as *mut SlotStr[V];
        memset(new_buf as *mut void, 0, new_size * sizeOf[SlotStr[V]]);
        self.data = new_buf;
        self.size = new_size;
        self.count = 0;
        let mut j: usize = 0;
        while j < old_size {
            let s: SlotStr[V] = old[j];
            if s.used && !s.deleted {
                let mut i: usize = s.hash % new_size;
                let mut steps: usize = 0;
                while steps < new_size && self.data[i].used {
                    i = (i + 1) % new_size;
                    steps = steps + 1;
                }
                self.data[i] = s;
                self.count = self.count + 1;
            }
            j = j + 1;
        }
        if old != null {
            free(old as *mut void);
        }
    }

    fn maybe_grow(&mut self) {
        if self.size > 0 && self.count * 4 >= self.size * 3 {
            self.grow_table(self.size * 2 + 1);
        }
    }

    fn put(&mut self, key: *const char, value: V) {
        self.maybe_grow();
        let size = self.size;
        let mut i: usize = hash_str(key) % size;
        let mut steps: usize = 0;
        while steps < size {
            let s: SlotStr[V] = self.data[i];
            if s.used {
                if !s.deleted && strcmp(s.key as *const char, key) == 0 {
                    let mut ns: SlotStr[V] = s;
                    ns.value = value;
                    self.data[i] = ns;
                    return;
                }
            } else {
                let mut ns: SlotStr[V] = s;
                ns.used = true;
                ns.deleted = false;
                ns.hash = hash_str(key);
                ns.key = dup_cstr(key);
                ns.value = value;
                self.data[i] = ns;
                self.count = self.count + 1;
                return;
            }
            i = (i + 1) % size;
            steps = steps + 1;
        }
        self.grow_table(size * 2 + 1);
        self.put(key, value);
    }

    fn find(&self, key: *const char) -> *mut SlotStr[V] {
        if self.size == 0 {
            return null;
        }
        let mut i: usize = hash_str(key) % self.size;
        let mut steps: usize = 0;
        while steps < self.size {
            let s: SlotStr[V] = self.data[i];
            if !s.used {
                return null;
            }
            if !s.deleted && strcmp(s.key as *const char, key) == 0 {
                return &mut self.data[i];
            }
            i = (i + 1) % self.size;
            steps = steps + 1;
        }
        return null;
    }

    fn contains(&self, key: *const char) -> bool {
        return self.find(key) != null;
    }

    fn remove(&mut self, key: *const char) -> bool {
        let slot = self.find(key);
        if slot == null {
            return false;
        }
        let mut s: SlotStr[V] = slot[0];
        free(s.key as *mut void);
        s.deleted = true;
        slot[0] = s;
        self.count = self.count - 1;
        return true;
    }

    fn keys(&self) -> Vec[*mut char] {
        let mut out: Vec[*mut char] = Vec::new();
        let mut j: usize = 0;
        while j < self.size {
            let s: SlotStr[V] = self.data[j];
            if s.used && !s.deleted {
                out.push(s.key);
            }
            j = j + 1;
        }
        return out;
    }

    fn clear(&mut self) {
        let mut j: usize = 0;
        while j < self.size {
            let s: SlotStr[V] = self.data[j];
            if s.used && !s.deleted {
                free(s.key as *mut void);
            }
            j = j + 1;
        }
        memset(self.data as *mut void, 0, self.size * sizeOf[SlotStr[V]]);
        self.count = 0;
    }

    fn free_buf(&self) {
        let mut j: usize = 0;
        while j < self.size {
            let s: SlotStr[V] = self.data[j];
            if s.used && !s.deleted && s.key != null {
                free(s.key as *mut void);
            }
            j = j + 1;
        }
        if self.data != null {
            free(self.data as *mut void);
        }
    }
}

struct HashMapI64[V] {
    data: *mut SlotI64[V],
    size: usize,
    count: usize,
}

impl HashMapI64[V] {
    fn new() -> HashMapI64[V] {
        let mut m: HashMapI64[V] = HashMapI64[V] { data: null, size: 0, count: 0 };
        m.grow_table(8);
        return m;
    }

    fn len(&self) -> usize {
        return self.count;
    }

    fn is_empty(&self) -> bool {
        return self.count == 0;
    }

    fn grow_table(&mut self, new_size: usize) {
        let old = self.data;
        let old_size = self.size;
        let new_buf = malloc(new_size * sizeOf[SlotI64[V]]) as *mut SlotI64[V];
        memset(new_buf as *mut void, 0, new_size * sizeOf[SlotI64[V]]);
        self.data = new_buf;
        self.size = new_size;
        self.count = 0;
        let mut j: usize = 0;
        while j < old_size {
            let s: SlotI64[V] = old[j];
            if s.used && !s.deleted {
                let mut i: usize = s.hash % new_size;
                let mut steps: usize = 0;
                while steps < new_size && self.data[i].used {
                    i = (i + 1) % new_size;
                    steps = steps + 1;
                }
                self.data[i] = s;
                self.count = self.count + 1;
            }
            j = j + 1;
        }
        if old != null {
            free(old as *mut void);
        }
    }

    fn maybe_grow(&mut self) {
        if self.size > 0 && self.count * 4 >= self.size * 3 {
            self.grow_table(self.size * 2 + 1);
        }
    }

    fn put(&mut self, key: i64, value: V) {
        self.maybe_grow();
        let size = self.size;
        let mut i: usize = hash_i64(key) % size;
        let mut steps: usize = 0;
        while steps < size {
            let s: SlotI64[V] = self.data[i];
            if s.used {
                if !s.deleted && s.key == key {
                    let mut ns: SlotI64[V] = s;
                    ns.value = value;
                    self.data[i] = ns;
                    return;
                }
            } else {
                let mut ns: SlotI64[V] = s;
                ns.used = true;
                ns.deleted = false;
                ns.hash = hash_i64(key);
                ns.key = key;
                ns.value = value;
                self.data[i] = ns;
                self.count = self.count + 1;
                return;
            }
            i = (i + 1) % size;
            steps = steps + 1;
        }
        self.grow_table(size * 2 + 1);
        self.put(key, value);
    }

    fn find(&self, key: i64) -> *mut SlotI64[V] {
        if self.size == 0 {
            return null;
        }
        let mut i: usize = hash_i64(key) % self.size;
        let mut steps: usize = 0;
        while steps < self.size {
            let s: SlotI64[V] = self.data[i];
            if !s.used {
                return null;
            }
            if !s.deleted && s.key == key {
                return &mut self.data[i];
            }
            i = (i + 1) % self.size;
            steps = steps + 1;
        }
        return null;
    }

    fn contains(&self, key: i64) -> bool {
        return self.find(key) != null;
    }

    fn remove(&mut self, key: i64) -> bool {
        let slot = self.find(key);
        if slot == null {
            return false;
        }
        let mut s: SlotI64[V] = slot[0];
        s.deleted = true;
        slot[0] = s;
        self.count = self.count - 1;
        return true;
    }

    fn keys(&self) -> Vec[i64] {
        let mut out: Vec[i64] = Vec::new();
        let mut j: usize = 0;
        while j < self.size {
            let s: SlotI64[V] = self.data[j];
            if s.used && !s.deleted {
                out.push(s.key);
            }
            j = j + 1;
        }
        return out;
    }

    fn clear(&mut self) {
        memset(self.data as *mut void, 0, self.size * sizeOf[SlotI64[V]]);
        self.count = 0;
    }

    fn free_buf(&self) {
        if self.data != null {
            free(self.data as *mut void);
        }
    }
}

struct HashSetStr {
    inner: HashMapStr[i32],
}

impl HashSetStr {
    fn new() -> HashSetStr {
        let mut s: HashSetStr = HashSetStr { inner: HashMapStr::new() };
        return s;
    }

    fn add(&mut self, key: *const char) {
        self.inner.put(key, 1);
    }

    fn contains(&self, key: *const char) -> bool {
        return self.inner.contains(key);
    }

    fn remove(&mut self, key: *const char) -> bool {
        return self.inner.remove(key);
    }

    fn len(&self) -> usize {
        return self.inner.len();
    }

    fn keys(&self) -> Vec[*mut char] {
        return self.inner.keys();
    }

    fn free_buf(&self) {
        self.inner.free_buf();
    }
}

struct HashSetI64 {
    inner: HashMapI64[i32],
}

impl HashSetI64 {
    fn new() -> HashSetI64 {
        let mut s: HashSetI64 = HashSetI64 { inner: HashMapI64::new() };
        return s;
    }

    fn add(&mut self, key: i64) {
        self.inner.put(key, 1);
    }

    fn contains(&self, key: i64) -> bool {
        return self.inner.contains(key);
    }

    fn remove(&mut self, key: i64) -> bool {
        return self.inner.remove(key);
    }

    fn len(&self) -> usize {
        return self.inner.len();
    }

    fn keys(&self) -> Vec[i64] {
        return self.inner.keys();
    }

    fn free_buf(&self) {
        self.inner.free_buf();
    }
}