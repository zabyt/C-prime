import std.io;

struct Point {
    x: i32,
    y: i32
}

impl Point {
    fn new(x: i32, y: i32) -> Point {
        return Point { x: x, y: y };
    }

    fn add(self, other: Point) -> Point {
        return Point { x: self.x + other.x, y: self.y + other.y };
    }

    fn scale(self, factor: i32) -> Point {
        return Point { x: self.x * factor, y: self.y * factor };
    }

    fn squared_length(self) -> i32 {
        return self.x * self.x + self.y * self.y;
    }
}

struct Color {
    r: i32,
    g: i32,
    b: i32
}

impl Color {
    fn new(r: i32, g: i32, b: i32) -> Color {
        return Color { r: r, g: g, b: b };
    }

    fn brighten(self, amount: i32) -> Color {
        return Color {
            r: self.r + amount,
            g: self.g + amount,
            b: self.b + amount
        };
    }

    fn darken(self, amount: i32) -> Color {
        return Color {
            r: self.r - amount,
            g: self.g - amount,
            b: self.b - amount
        };
    }
}

fn add_i32(a: i32, b: i32) -> i32 {
    return a + b;
}

fn subtract_i32(a: i32, b: i32) -> i32 {
    return a - b;
}

fn multiply_i32(a: i32, b: i32) -> i32 {
    return a * b;
}

fn divide_i32(a: i32, b: i32) -> i32 {
    return a / b;
}

fn max_i32(a: i32, b: i32) -> i32 {
    if a > b {
        return a;
    }
    return b;
}

fn min_i32(a: i32, b: i32) -> i32 {
    if a < b {
        return a;
    }
    return b;
}

fn abs_i32(value: i32) -> i32 {
    if value < 0 {
        return -value;
    }
    return value;
}

fn is_even(value: i32) -> bool {
    if value % 2 == 0 {
        return true;
    }
    return false;
}

fn is_odd(value: i32) -> bool {
    if value % 2 != 0 {
        return true;
    }
    return false;
}

fn clamp(value: i32, minv: i32, maxv: i32) -> i32 {
    if value < minv {
        return minv;
    }
    if value > maxv {
        return maxv;
    }
    return value;
}

fn sum_up_to(limit: i32) -> i32 {
    let mut total: i32 = 0;
    let mut i: i32 = 0;
    while i <= limit {
        total = total + i;
        i = i + 1;
    }
    return total;
}

fn factorial(n: i32) -> i32 {
    let mut result: i32 = 1;
    let mut i: i32 = 2;
    while i <= n {
        result = result * i;
        i = i + 1;
    }
    return result;
}

fn fibonacci(n: i32) -> i32 {
    if n <= 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }

    let mut a: i32 = 0;
    let mut b: i32 = 1;
    let mut i: i32 = 2;
    while i <= n {
        let next: i32 = a + b;
        a = b;
        b = next;
        i = i + 1;
    }
    return b;
}

fn count_digits(value: i32) -> i32 {
    let mut number: i32 = abs_i32(value);
    let mut count: i32 = 0;
    if number == 0 {
        return 1;
    }
    while number > 0 {
        count = count + 1;
        number = number / 10;
    }
    return count;
}

fn grade_to_text(score: i32) -> i32 {
    if score >= 90 {
        return 5;
    }
    if score >= 75 {
        return 4;
    }
    if score >= 60 {
        return 3;
    }
    if score >= 50 {
        return 2;
    }
    return 1;
}

fn choose_message(level: i32) -> i32 {
    match level {
        1 => return 1,
        2 => return 2,
        3 => return 3,
        4 => return 4,
        _ => return 5,
    }
}

fn generic_identity[T](value: T) -> T {
    return value;
}

fn pointer_read[T](ptr: *const T) -> T {
    return *ptr;
}

fn pointer_write[T](ptr: *mut T, value: T) {
    *ptr = value;
    return;
}

fn main() -> i32 {
    let first: i32 = 5;
    let second: i32 = 9;
    let third: i32 = 12;
    let fourth: i32 = -7;

    let sum1: i32 = add_i32(first, second);
    let sum2: i32 = subtract_i32(third, first);
    let product: i32 = multiply_i32(sum1, sum2);
    let quotient: i32 = divide_i32(product, 2);
    let maxv: i32 = max_i32(sum1, product);
    let minv: i32 = min_i32(sum2, quotient);
    let absolute: i32 = abs_i32(fourth);
    let clamped: i32 = clamp(42, 10, 30);
    let total_to_10: i32 = sum_up_to(10);
    let fact5: i32 = factorial(5);
    let fib8: i32 = fibonacci(8);
    let digits: i32 = count_digits(-12345);
    let grade: i32 = grade_to_text(82);
    let level: i32 = choose_message(grade);

    let mut x: i32 = 0;
    let mut y: i32 = 1;
    let mut z: i32 = 2;
    let mut counter: i32 = 0;

    while counter < 10 {
        x = x + counter;
        y = y + 2;
        z = z + 3;
        counter = counter + 1;
    }

    if x > y {
        println("x is larger");
    } else {
        println("y is larger or equal");
    }

    if is_even(z) {
        println("z is even");
    } else {
        println("z is odd");
    }

    if is_odd(x) {
        println("x is odd");
    } else {
        println("x is even");
    }

    let mut i: i32 = 0;
    while i < 5 {
        if i % 2 == 0 {
            println(i);
        } else {
            println(i + 100);
        }
        i = i + 1;
    }

    let origin: Point = Point::new(0, 0);
    let p1: Point = Point::new(2, 3);
    let p2: Point = Point::new(4, 1);
    let p3: Point = p1.add(p2).scale(2);
    let p4: Point = Point { x: 10, y: 20 };
    let dist: i32 = p3.squared_length();
    let total_point: i32 = origin.x + p4.y + dist;

    let c1: Color = Color::new(10, 20, 30);
    let c2: Color = c1.brighten(5).darken(2);
    let c3: Color = Color { r: 200, g: 180, b: 160 };

    let mut ptr_value: i32 = 123;
    let ptr: *mut i32 = &mut ptr_value;
    pointer_write(ptr, 456);
    let read_back: i32 = pointer_read(ptr);

    let identity_i32: i32 = generic_identity(77);
    let identity_bool: bool = generic_identity(true);

    match level {
        1 => println("weak"),
        2 => println("fair"),
        3 => println("good"),
        4 => println("very good"),
        _ => println("excellent"),
    }

    println(sum1);
    println(sum2);
    println(product);
    println(quotient);
    println(maxv);
    println(minv);
    println(absolute);
    println(clamped);
    println(total_to_10);
    println(fact5);
    println(fib8);
    println(digits);
    println(grade);
    println(level);
    println(x);
    println(y);
    println(z);
    println(counter);
    println(p3.x);
    println(p3.y);
    println(p4.x);
    println(p4.y);
    println(total_point);
    println(c2.r);
    println(c2.g);
    println(c2.b);
    println(c3.r);
    println(c3.g);
    println(c3.b);
    println(ptr_value);
    println(read_back);
    println(identity_i32);
    println(identity_bool);

    let mut running_total: i32 = 0;
    let mut j: i32 = 0;
    while j < 20 {
        running_total = running_total + j;
        if j % 3 == 0 {
            running_total = running_total + 10;
        }
        if j % 5 == 0 {
            running_total = running_total + 5;
        }
        j = j + 1;
    }

    let mut k: i32 = 0;
    while k < 8 {
        let next_value: i32 = k * 3;
        if next_value > 10 {
            println(next_value);
        } else {
            println(k);
        }
        k = k + 1;
    }

    let mut sum_loop: i32 = 0;
    let mut n: i32 = 0;
    while n < 12 {
        sum_loop = sum_loop + n;
        n = n + 1;
    }

    let bool_value: bool = true;
    let bool_other: bool = false;
    let bool_result: bool = bool_value && !bool_other;

    if bool_result {
        println(1);
    } else {
        println(0);
    }

    let mut count_a: i32 = 0;
    let mut count_b: i32 = 0;
    let mut count_c: i32 = 0;

    while count_a < 5 {
        count_a = count_a + 1;
        count_b = count_b + 2;
        count_c = count_c + 3;
    }

    let final1: i32 = max_i32(count_a, count_b);
    let final2: i32 = min_i32(count_b, count_c);
    let final3: i32 = abs_i32(-final1 + final2);
    let final_total: i32 = final1 + final2 + final3 + running_total + sum_loop;

    println(final1);
    println(final2);
    println(final3);
    println(final_total);

    let mut m: i32 = 0;
    while m < 6 {
        let value: i32 = m * 4;
        println(value);
        m = m + 1;
    }

    for ch in "HELLO" {
        println(ch);
    }

    let mut s: i32 = 100;
    let mut t: i32 = 0;
    while t < 5 {
        s = s - 3;
        t = t + 1;
    }

    let q1: i32 = s + x + y + z;
    let q2: i32 = q1 / 2;
    let q3: i32 = q2 + fact5;
    let q4: i32 = q3 - fib8;
    let q5: i32 = q4 + total_to_10;

    if q5 >= 0 {
        println(q5);
    } else {
        println(0);
    }

    let mut u: i32 = 0;
    let mut v: i32 = 0;
    while u < 9 {
        v = v + u;
        u = u + 1;
    }

    let result_block: i32 = v + q5 + count_a + count_b + count_c;
    println(result_block);

    return 0;
}
