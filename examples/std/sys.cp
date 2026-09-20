extern fn getenv(name: *const char) -> *const char;
extern fn time(timer: *mut i64) -> i64;
extern fn Sleep(ms: u32) -> void;
extern fn GetCommandLineA() -> *const char;
extern fn malloc(size: usize) -> *mut void;
extern fn free(ptr: *mut void) -> void;
extern fn fopen(path: *const char, mode: *const char) -> *mut void;
extern fn fclose(f: *mut void) -> i32;
extern fn fseek(f: *mut void, off: i64, origin: i32) -> i32;
extern fn ftell(f: *mut void) -> i64;
extern fn fread(dst: *mut void, size: usize, count: usize, f: *mut void) -> usize;
extern fn fwrite(src: *const void, size: usize, count: usize, f: *mut void) -> usize;

fn env_get(name: *const char) -> *const char {
    return getenv(name);
}

fn time_s() -> i64 {
    return time(null);
}

fn sleep_ms(ms: i64) {
    if ms > 0 {
        Sleep(ms as u32);
    }
}

fn cmd_line() -> *const char {
    return GetCommandLineA();
}

fn read_file_all(path: *const char) -> *mut char {
    let f = fopen(path, "rb");
    if f == null {
        return null;
    }
    fseek(f, 0, 2 as i32);
    let n = ftell(f);
    fseek(f, 0, 0 as i32);
    if n <= 0 {
        fclose(f);
        return null;
    }
    let buf = malloc(n as usize + 1) as *mut char;
    fread(buf as *mut void, 1, n as usize, f);
    buf[n as usize] = 0 as char;
    fclose(f);
    return buf;
}

fn write_file_all(path: *const char, data: *const char, n: usize) -> i32 {
    let f = fopen(path, "wb");
    if f == null {
        return -1;
    }
    fwrite(data as *const void, 1, n, f);
    fclose(f);
    return 0;
}

struct Rng {
    state: i64,
}

impl Rng {
    fn new(seed: i64) -> Rng {
        let mut s: Rng = Rng { state: 0 };
        s.state = seed;
        if s.state == 0 {
            s.state = 88172645463325252;
        }
        return s;
    }

    fn next(&mut self) -> i64 {
        let mut x = self.state;
        x = x ^ (x >> 12);
        x = x ^ (x << 25);
        x = x ^ (x >> 27);
        self.state = x;
        return x * 2685821657736338717;
    }

    fn range(&mut self, lo: i64, hi: i64) -> i64 {
        let span = hi - lo;
        if span <= 0 {
            return lo;
        }
        let mut r = self.next();
        if r < 0 {
            r = 0 - r;
        }
        return lo + r % span;
    }

    fn chance(&mut self, num: i64, den: i64) -> bool {
        if den <= 0 {
            return false;
        }
        if num <= 0 {
            return false;
        }
        if num >= den {
            return true;
        }
        return self.range(0, den) < num;
    }
}