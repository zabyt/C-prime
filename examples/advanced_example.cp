struct Vec2 {
    x: i32,
    y: i32
}

impl Vec2 {
    fn new(x: i32, y: i32) -> Vec2 {
        return Vec2 { x: x, y: y };
    }

    fn add(self, other: Vec2) -> Vec2 {
        return Vec2 { x: self.x + other.x, y: self.y + other.y };
    }
}

fn square(value: i32) -> i32 {
    return value * value;
}

fn main() -> i32 {
    let mut total: i32 = 0;
    let mut i: i32 = 0;

    while i < 5 {
        total = total + i;
        i = i + 1;
    }

    let a: Vec2 = Vec2::new(2, 3);
    let b: Vec2 = Vec2::new(4, 5);
    let c: Vec2 = a.add(b);

    println("total:");
    println(total);
    println("vector:");
    println(c.x);
    println(c.y);

    let result: i32 = square(total + c.x + c.y);
    println(result);

    match result {
        1 => println("small"),
        2 => println("tiny"),
        3 => println("low"),
        _ => println("other"),
    }

    return 0;
}
