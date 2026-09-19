include "std/ctypes.cp";

fn ftos(v: f64, buf: *mut char) -> i32 {
    let mut x = v;
    let mut i = 0;
    if x < 0.0 {
        buf[0] = '-';
        i = 1;
        x = -x;
    }
    let mut ip: f64 = 0.0;
    let mut fp = modf(x, &mut ip);
    let mut n = ip as u64;
    if n == 0 {
        buf[i] = '0';
        i = i + 1;
    }
    while n > 0 {
        let d = (n % 10) as i32;
        n = n / 10;
        buf[i] = (48 + d) as char;
        i = i + 1;
    }
    let mut l = 0;
    let mut r = i - 1;
    while l < r {
        let t = buf[l];
        buf[l] = buf[r];
        buf[r] = t;
        l = l + 1;
        r = r - 1;
    }
    if fp > 0.0 {
        let mut frac = floor(fp * 100000000.0 + 0.5) as u64;
        if frac > 0 {
            buf[i] = '.';
            i = i + 1;
            let start = i;
            let mut w = 0;
            while w < 8 {
                buf[i] = (48 + (frac % 10) as i32) as char;
                i = i + 1;
                frac = frac / 10;
                w = w + 1;
            }
            let mut l = start;
            let mut r = i - 1;
            while l < r {
                let t = buf[l];
                buf[l] = buf[r];
                buf[r] = t;
                l = l + 1;
                r = r - 1;
            }
            while i > start + 1 && buf[i - 1] == '0' {
                i = i - 1;
            }
        }
    }
    buf[i] = 0 as char;
    return i;
}

fn show(v: f64, buf: *mut char) {
    let n = ftos(v, buf);
    let mut i = 0;
    while i < n {
        let c = buf[i];
        if (c as i32) == 0 {
            break;
        }
        println(c as i32);
        i = i + 1;
    }
    println(-1);
}

fn main() {
    let buf: *mut char = malloc(40) as *mut char;
    show(0.0, buf);
    show(3.14, buf);
    show(123456789.0, buf);
    show(-42.5, buf);
    show(1000000.0, buf);
    show(0.000000123, buf);
    show(99999999.99, buf);
}
