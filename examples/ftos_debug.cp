include "std/ctypes.cp";

fn main() {
    let buf: *mut char = malloc(40) as *mut char;
    let mut x = 3.14;
    let mut ip: f64 = 0.0;
    let mut fp = modf(x, &mut ip);
    println(ip);
    println(fp);
    let mut n = ip as u64;
    println(n);
    while n > 0 {
        let d = (n % 10) as i32;
        n = n / 10;
        println(d);
    }
    let mut k = 0;
    while k < 8 && fp > 0.0 {
        fp = fp * 10.0;
        let d = fp as i32;
        fp = fp - (d as f64);
        println(d);
        println(fp);
        k = k + 1;
    }
    println(k);
}
