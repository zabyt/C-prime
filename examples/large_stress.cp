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

    fn length(self) -> f64 {
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

    fn lighten(self, amount: i32) -> Color {
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

struct Stats {
    hp: i32,
    mana: i32,
    level: i32
}

impl Stats {
    fn new(hp: i32, mana: i32, level: i32) -> Stats {
        return Stats { hp: hp, mana: mana, level: level };
    }

    fn heal(self, amount: i32) -> Stats {
        return Stats {
            hp: self.hp + amount,
            mana: self.mana,
            level: self.level
        };
    }

    fn spend_mana(self, amount: i32) -> Stats {
        return Stats {
            hp: self.hp,
            mana: self.mana - amount,
            level: self.level
        };
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

fn is_even(value: i32) -> bool {
    if value % 2 == 0 {
        return true;
    }
    return false;
}

fn increment_until(limit: i32) -> i32 {
    let mut i: i32 = 0;
    while i < limit {
        i = i + 1;
    }
    return i;
}

fn compute_score(base: i32, bonus: i32, multiplier: i32) -> i32 {
    let mut total: i32 = base + bonus;
    let mut i: i32 = 0;
    while i < multiplier {
        total = total + 2;
        i = i + 1;
    }
    return total;
}

fn swap[T](a: *mut T, b: *mut T) {
    let tmp: T = *a;
    *a = *b;
    *b = tmp;
    return;
}

fn choose_message(score: i32) -> i32 {
    if score < 10 {
        return 1;
    }
    if score < 20 {
        return 2;
    }
    if score < 30 {
        return 3;
    }
    if score < 40 {
        return 4;
    }
    if score < 50 {
        return 5;
    }
    return 6;
}

fn run_block_1() -> i32 {
    let mut a: i32 = 0;
    let mut b: i32 = 1;
    let mut c: i32 = 2;
    while a < 5 {
        b = b + 2;
        c = c + 3;
        a = a + 1;
    }
    return a + b + c;
}

fn run_block_2() -> i32 {
    let mut x: i32 = 7;
    let mut y: i32 = 9;
    let mut z: i32 = 11;
    if x < y {
        z = z + 1;
    } else {
        z = z + 2;
    }
    if z > 10 {
        y = y + 3;
    }
    while x < 15 {
        x = x + 2;
        y = y + 1;
    }
    return x + y + z;
}

fn run_block_3() -> i32 {
    let mut total: i32 = 0;
    let mut i: i32 = 0;
    while i < 10 {
        if is_even(i) {
            total = total + i;
        } else {
            total = total + i * 2;
        }
        i = i + 1;
    }
    return total;
}

fn run_block_4() -> i32 {
    let mut v: i32 = 12;
    let mut w: i32 = 4;
    let mut r: i32 = 0;
    if v > w {
        r = r + 1;
    }
    if v == 12 {
        r = r + 2;
    }
    if w < 5 {
        r = r + 3;
    }
    return r;
}

fn run_block_5() -> i32 {
    let mut i: i32 = 0;
    let mut sum: i32 = 0;
    while i < 8 {
        sum = sum + i;
        i = i + 1;
    }
    return sum;
}

fn run_block_6() -> i32 {
    let mut x: i32 = 5;
    let mut y: i32 = 6;
    let mut z: i32 = 7;
    if x < y {
        z = z + 4;
    }
    if y > 5 {
        z = z + 5;
    }
    return z;
}

fn run_block_7() -> i32 {
    let mut value: i32 = 1;
    let mut total: i32 = 0;
    while value < 10 {
        total = total + value;
        value = value + 1;
    }
    return total;
}

fn run_block_8() -> i32 {
    let mut value: i32 = 20;
    let mut result: i32 = 0;
    while value > 0 {
        result = result + value;
        value = value - 3;
    }
    return result;
}

fn main() -> i32 {
    let p1: Point = Point::new(1.5, 2.5);
    let p2: Point = Point::new(3.0, 4.0);
    let p3: Point = p1.add(p2).scale(2.0);
    let per: f64 = p3.x + p3.y;
    let bounded: f64 = clamp(per, 0.0, 50.0);
    let mut score: i32 = 0;

    let c1: Color = Color::new(10, 20, 30);
    let c2: Color = c1.lighten(5).darken(2);
    let c3: Color = Color::new(200, 180, 160);
    let s1: Stats = Stats::new(100, 50, 4);
    let s2: Stats = s1.heal(10).spend_mana(5);

    let mut a: i32 = 1;
    let mut b: i32 = 2;
    let mut c: i32 = 3;
    let mut d: i32 = 4;
    let mut e: i32 = 5;

    if a < b {
        score = score + 1;
    }
    if b < c {
        score = score + 2;
    }
    if c < d {
        score = score + 3;
    }
    if d < e {
        score = score + 4;
    }

    let mut i: i32 = 0;
    while i < 10 {
        score = score + i;
        i = i + 1;
    }

    let total_block: i32 = run_block_1() + run_block_2() + run_block_3() + run_block_4() + run_block_5() + run_block_6() + run_block_7() + run_block_8();
    score = score + total_block;

    let msg: i32 = choose_message(score);
    let limit: i32 = max_i32(c2.r, c3.r);
    let floor: i32 = min_i32(c2.g, c3.g);
    let even_check: bool = is_even(score);

    match msg {
        1 => println("low"),
        2 => println("small"),
        3 => println("medium"),
        4 => println("higher"),
        5 => println("strong"),
        _ => println("peak"),
    }

    if even_check {
        println(limit);
    } else {
        println(floor);
    }

    println(bounded);
    println(score);
    println(s2.hp);
    println(s2.mana);
    println(c2.r);
    println(c2.g);
    println(c2.b);

    let mut x: i32 = 0;
    let mut y: i32 = 1;
    let mut z: i32 = 2;
    let mut step: i32 = 0;

    while step < 7 {
        x = x + y;
        y = y + z;
        z = z + 1;
        step = step + 1;
    }

    let mut w: i32 = 0;
    let mut v: i32 = 30;
    while w < 5 {
        v = v - 2;
        w = w + 1;
    }

    let outer: i32 = compute_score(10, 8, 4);
    let inner: i32 = increment_until(9);
    let result: i32 = outer + inner + x + y + z + v;

    let left_value: i32 = max_i32(result, 50);
    let right_value: i32 = min_i32(result, 100);
    let final_value: i32 = left_value + right_value;

    println(final_value);

    for ch in "C-Prime" {
        println(ch);
    }

    let mut temp: i32 = 1;
    let mut loop_count: i32 = 0;
    while loop_count < 6 {
        temp = temp * 2;
        loop_count = loop_count + 1;
    }

    let mut first: i32 = 10;
    let mut second: i32 = 20;
    let mut third: i32 = 30;
    swap(&mut first, &mut second);
    swap(&mut second, &mut third);

    if first > second {
        println(first);
    } else {
        println(second);
    }

    if third > first {
        println(third);
    } else {
        println(first);
    }

    println(temp);
    println(loop_count);
    println(result);
    println(final_value);

    return 0;
}
