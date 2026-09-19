import std.io;

struct Vector2 {
    x: f64,
    y: f64
}

impl Vector2 {
    fn new(x: f64, y: f64) -> Vector2 {
        return Vector2 { x: x, y: y };
    }

    fn length(self) -> f64 {
        return self.x * self.x + self.y * self.y;
    }
}

fn swap[T](a: *mut T, b: *mut T) {
    let tmp: T = *a;
    *a = *b;
    *b = tmp;
    return;
}

fn main() -> i32 {
    let v: Vector2 = Vector2::new(1.5, 2.5);
    let mut total: f64 = v.length();

    if total > 0.0 {
        println(total);
    } else {
        println(0.0);
    }

    let mut i: i32 = 0;
    while i < 3 {
        total = total + 1.0;
        i = i + 1;
    }

    match i {
        3 => println("match ok"),
        _ => println("fallback"),
    }

    for ch in "CP" {
        println(ch);
    }

    return 0;
}
