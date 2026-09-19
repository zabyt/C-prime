import std.io;

fn main() -> i32 {
    println("hello from C-Prime");
    let mut sum: i32 = 0;
    let mut i: i32 = 0;
    while i < 10 {
        sum = sum + i;
        i = i + 1;
    }
    println("sum =");
    println(sum);

    let score: i32 = 5;
    switch score {
        case 3:
            println("three");
            break;
        case 5:
            println("five");
            break;
        default:
            println("other");
    }

    let mut j: i32 = 0;
    while j < 10 {
        j = j + 1;
        if j == 3 {
            continue;
        }
        if j == 6 {
            break;
        }
        println(j);
    }
    return 0;
}
