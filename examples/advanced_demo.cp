import std.io;

struct Point {
    x: f64,
    y: f64
}

impl Point {
    fn new(x: f64, y: f64) -> Point {
        return Point { x: x, y: y };
    }

    fn add(self, other: Point) -> Point {
        return Point { x: self.x + other.x, y: self.y + other.y };
    }

    fn scale(self, factor: f64) -> Point {
        return Point { x: self.x * factor, y: self.y * factor };
    }
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    return value;
}

fn swap[T](a: *mut T, b: *mut T) {
    let tmp: T = *a;
    *a = *b;
    *b = tmp;
    return;
}

fn main() -> i32 {
    let p1: Point = Point::new(1.5, 2.0);
    let p2: Point = Point::new(2.0, 3.5);
    let p3: Point = p1.add(p2).scale(2.0);

    let mut total: f64 = p3.x + p3.y;
    let mut i: i32 = 0;

    while i < 4 {
        total = total + 1.25;
        i = i + 1;
    }

    let bounded: f64 = clamp(total, 0.0, 20.0);
    println(bounded);

    match i {
        4 => println("loop done"),
        _ => println("other"),
    }

    for ch in "C-Prime" {
        println(ch);
    }

    return 0;
}
