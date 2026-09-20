








include "vec.cp";
extern fn malloc(size: usize) -> *mut void;
extern fn free(ptr: *mut void) -> void;
extern fn strlen(s: *const char) -> usize;
extern fn strcmp(a: *const char, b: *const char) -> i32;
extern fn strcpy(dst: *mut char, src: *const char) -> *mut char;

struct IntMap {
    keys: Vec[i64],
    values: Vec[i64],
}

impl IntMap {
    fn new() -> IntMap {
        let keys: Vec[i64] = Vec::new();
        let values: Vec[i64] = Vec::new();
        return IntMap { keys: keys, values: values };
    }

    fn len(&self) -> usize {
        return self.keys.len();
    }

    fn is_empty(&self) -> bool {
        return self.keys.is_empty();
    }

    fn contains(&self, k: i64) -> bool {
        let mut i: usize = 0;
        while i < self.keys.len() {
            let ki: i64 = self.keys.get(i);
            if ki == k {
                return true;
            }
            i = i + 1;
        }
        return false;
    }

    fn get(&self, k: i64) -> i64 {
        let mut i: usize = 0;
        while i < self.keys.len() {
            let ki: i64 = self.keys.get(i);
            if ki == k {
                let v: i64 = self.values.get(i);
                return v;
            }
            i = i + 1;
        }
        return 0;
    }

    fn set(&mut self, k: i64, v: i64) {
        let mut idx: usize = self.keys.len();
        let mut i: usize = 0;
        while i < self.keys.len() {
            let ki: i64 = self.keys.get(i);
            if ki == k {
                idx = i;
                i = self.keys.len();
            } else {
                i = i + 1;
            }
        }
        if idx < self.keys.len() {
            self.values.set(idx, v);
            return;
        }
        self.keys.push(k);
        self.values.push(v);
    }

    fn remove(&mut self, k: i64) {
        let mut idx: usize = self.keys.len();
        let mut i: usize = 0;
        while i < self.keys.len() {
            let ki: i64 = self.keys.get(i);
            if ki == k {
                idx = i;
                i = self.keys.len();
            } else {
                i = i + 1;
            }
        }
        if idx == self.keys.len() {
            return;
        }
        let mut j: usize = idx;
        while j + 1 < self.keys.len() {
            self.keys.set(j, self.keys.get(j + 1));
            self.values.set(j, self.values.get(j + 1));
            j = j + 1;
        }
        self.keys.truncate(self.keys.len() - 1);
        self.values.truncate(self.values.len() - 1);
    }

    fn clear(&mut self) {
        self.keys.truncate(0);
        self.values.truncate(0);
    }

    fn free_buf(&self) {
        self.keys.free_buf();
        self.values.free_buf();
    }
}





struct StrIntMap {
    keys: Vec[*mut char],
    values: Vec[i64],
}

impl StrIntMap {
    fn new() -> StrIntMap {
        let keys: Vec[*mut char] = Vec::new();
        let values: Vec[i64] = Vec::new();
        return StrIntMap { keys: keys, values: values };
    }

    fn len(&self) -> usize {
        return self.keys.len();
    }

    fn find(&self, k: *const char) -> i64 {
        let mut i: usize = 0;
        while i < self.keys.len() {
            let ki: *mut char = self.keys.get(i);
            if strcmp(ki, k) == 0 {
                return i as i64;
            }
            i = i + 1;
        }
        return -1;
    }

    fn increment(&mut self, k: *const char) {
        let idx: i64 = self.find(k);
        if idx >= 0 {
            self.values.set(idx as usize, self.values.get(idx as usize) + 1);
            return;
        }
        let n: usize = strlen(k);
        let copy = malloc(n + 1) as *mut char;
        strcpy(copy, k);
        self.keys.push(copy);
        self.values.push(1);
    }

    fn get(&self, k: *const char) -> i64 {
        let idx: i64 = self.find(k);
        if idx >= 0 {
            return self.values.get(idx as usize);
        }
        return 0;
    }

    fn key(&self, i: usize) -> *const char {
        return self.keys.get(i) as *const char;
    }

    fn count(&self, i: usize) -> i64 {
        return self.values.get(i);
    }

    fn free_buf(&self) {
        let mut i: usize = 0;
        while i < self.keys.len() {
            free(self.keys.get(i) as *mut void);
            i = i + 1;
        }
        self.keys.free_buf();
        self.values.free_buf();
    }
}