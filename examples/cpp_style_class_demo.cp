class Vector2 {
    x: f64,
    y: f64,

    fn new(x: f64, y: f64) -> Vector2 {
        return Vector2 { x: x, y: y };
    }

    fn length(self) -> f64 {
        return self.x + self.y;
    }

    fn scale(self, factor: f64) -> Vector2 {
        return Vector2 { x: self.x * factor, y: self.y * factor };
    }
}

fn main() -> i32 {
    let v: Vector2 = Vector2::new(1.5, 2.0);
    let l: f64 = v.length();
    let scaled: Vector2 = v.scale(2.0);

    println("C++-style class API demo");
    println(l);
    println(scaled.x);
    return 0;
}
