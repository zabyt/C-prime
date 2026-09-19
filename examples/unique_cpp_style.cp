import std.io;

struct Vector3 {
    x: f64,
    y: f64,
    z: f64
}

impl Vector3 {
    fn new(x: f64, y: f64, z: f64) -> Vector3 {
        return Vector3 { x: x, y: y, z: z };
    }

    fn add(self, other: Vector3) -> Vector3 {
        return Vector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z
        };
    }

    fn scale(self, factor: f64) -> Vector3 {
        return Vector3 {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor
        };
    }

    fn length(self) -> f64 {
        return self.x * self.x + self.y * self.y + self.z * self.z;
    }
}

fn clamp(value: f64, min_value: f64, max_value: f64) -> f64 {
    if value < min_value {
        return min_value;
    } else if value > max_value {
        return max_value;
    } else {
        return value;
    }
}

fn main() -> i32 {
    let a: Vector3 = Vector3::new(1.0, 2.0, 3.0);
    let b: Vector3 = Vector3::new(0.5, 1.0, 1.5);
    let sum: Vector3 = a.add(b);
    let length: f64 = sum.length();
    let safe_length: f64 = clamp(length, 0.0, 25.0);

    println("C-Prime: a unique systems style");
    println("vector length =");
    println(safe_length);

    match safe_length {
        0.0 => println("empty vector"),
        _ => println("vector is valid"),
    }

    return 0;
}
