include "std/ctypes.cp";

fn main() {
    let a = 0.0;
    let b = 0.0;
    if a > b {
        println(1);
    } else {
        println(2);
    }
    let c = 3.0;
    let d = 2.0;
    if c > d {
        println(3);
    } else {
        println(4);
    }
    let mut k = 2;
    while k < 8 && a > b {
        println(k);
        k = k + 1;
    }
    println(99);
}
