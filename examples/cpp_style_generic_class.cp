class Box[T] {
    value: T,

    fn new(v: T) -> Box[T] {
        return Box[T] { value: v };
    }

    fn get(self) -> T {
        return self.value;
    }

    fn set(self, v: T) -> Box[T] {
        self.value = v;
        return self;
    }
}

fn main() -> i32 {
    let a: Box[i32] = Box { value: 7 };
    let b: Box[i32] = Box::new(5);
    let v: i32 = b.get();
    let changed: Box[i32] = a.set(v);

    println("generic class demo");
    println(changed.get());
    return 0;
}
