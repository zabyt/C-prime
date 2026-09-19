import std.io;

fn add(a: f64, b: f64) -> f64 {
    return a + b;
}

fn subtract(a: f64, b: f64) -> f64 {
    return a - b;
}

fn multiply(a: f64, b: f64) -> f64 {
    return a * b;
}

fn divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        println("division by zero");
        return 0.0;
    }
    return a / b;
}

fn main() -> i32 {
    let x: f64 = 12.5;
    let y: f64 = 3.0;

    println("C-Prime calculator");
    println("x = 12.5");
    println("y = 3.0");
    println("sum =");
    println(add(x, y));
    println("difference =");
    println(subtract(x, y));
    println("product =");
    println(multiply(x, y));
    println("quotient =");
    println(divide(x, y));

    return 0;
}
