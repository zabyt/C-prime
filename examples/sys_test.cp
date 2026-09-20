include "std/sys.cp";

extern fn printf(fmt: *const char, ...) -> i32;
extern fn strlen(s: *const char) -> usize;

fn main() -> i32 {
    let t = time_s();
    printf("seconds since epoch: %lld\n", t);

    let mut r = Rng::new(12345);
    let mut i: i64 = 0;
    let mut sum: i64 = 0;
    while i < 1000 {
        sum = sum + r.range(0, 100);
        i = i + 1;
    }
    printf("average of 1000 draws in [0,100): %lld\n", sum / 1000);

    let mut a = Rng::new(12345);
    let mut b = Rng::new(12345);
    if a.next() != b.next() {
        return 1;
    }
    let mut d = Rng::new(12345);
    let v1 = d.next();
    if r.next() == v1 {
        return 2;
    }

    let path = env_get("PATH");
    if path == null {
        printf("PATH is not set\n");
    } else {
        printf("PATH is set\n");
    }

    let big: *mut char = malloc(1000) as *mut char;
    let mut k: i64 = 0;
    while k < 1000 {
        big[k as usize] = 'x' as char;
        k = k + 1;
    }
    if write_file_all("_big_out.txt", big, 1000) != 0 {
        return 3;
    }
    let bg = read_file_all("_big_out.txt");
    if bg == null {
        return 3;
    }
    if strlen(bg) != 1000 {
        return 3;
    }
    printf("read %lld chars from _big_out.txt\n", strlen(bg) as i64);
    free(bg as *mut void);

    let w = write_file_all("_sys_out.txt", "hello", 5);
    if w != 0 {
        return 4;
    }
    let back = read_file_all("_sys_out.txt");
    if back == null {
        return 5;
    }
    printf("wrote and read back: %s\n", back);
    free(back as *mut void);

    sleep_ms(20);
    printf("slept 20ms\n");
    return 0;
}