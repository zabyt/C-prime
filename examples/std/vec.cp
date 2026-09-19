









extern fn malloc(size: usize) -> *mut void;
extern fn free(ptr: *mut void) -> void;
extern fn realloc(ptr: *mut void, size: usize) -> *mut void;

struct Vec[T] {
    data: *mut T,
    length: usize,
    capacity: usize,
}

impl Vec[T] {
    fn new() -> Vec[T] {
        return Vec[T] { data: null, length: 0, capacity: 0 };
    }

    fn with_capacity(cap: usize) -> Vec[T] {
        let mut v: Vec[T] = Vec::new();
        v.reserve(cap);
        return v;
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

    fn get(&self, i: usize) -> T {
        return self.data[i];
    }

    fn set(&mut self, i: usize, value: T) {
        self.data[i] = value;
    }

    fn push(&mut self, value: T) {
        if self.length == self.capacity {
            self.grow(self.capacity * 2 + 4);
        }
        self.data[self.length] = value;
        self.length = self.length + 1;
    }

    fn pop(&mut self) -> T {
        if self.length > 0 {
            self.length = self.length - 1;
            return self.data[self.length];
        }
        return self.data[0];
    }

    fn reserve(&mut self, extra: usize) {
        let required = self.length + extra;
        if required > self.capacity {
            let mut new_cap = required;
            if self.capacity * 2 > new_cap {
                new_cap = self.capacity * 2;
            }
            self.grow(new_cap);
        }
    }

    fn grow(&mut self, new_cap: usize) {
        let bytes: usize = sizeOf[T] * new_cap;
        if self.data == null {
            self.data = malloc(bytes) as *mut T;
        } else {
            self.data = realloc(self.data as *mut void, bytes) as *mut T;
        }
        self.capacity = new_cap;
    }

    fn truncate(&mut self, n: usize) {
        if n < self.length {
            self.length = n;
        }
    }

    fn clear(&mut self) {
        self.length = 0;
    }

    fn free_buf(&self) {
        free(self.data as *mut void);
    }
}