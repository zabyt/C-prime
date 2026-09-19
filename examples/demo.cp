import std.io;

struct Vector2 {
    x: f64,
    y: f64
}

impl Vector2 {
    fn new(x: f64, y: f64) -> Vector2 {
        return Vector2 { x: x, y: y };
    }
}

fn main() -> i32 {
    let v: Vector2 = Vector2::new(1.5, 2.5);
    match 0x2A {
        42 => println(v.x),
        _ => return 1,
    }
    return 0;
}
