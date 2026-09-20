include "../std/sys.cp";
include "../std/string.cp";

extern fn getchar() -> i32;

fn read_line() -> String {
    let mut s: String = String::new();
    while true {
        let c: i32 = getchar();
        if c < 0 || c == 10 {
            break;
        }
        s.push(c as char);
    }
    return s;
}

fn main() -> i32 {
    let mut rng: Rng = Rng::new(time_s());
    let secret: i64 = rng.range(1, 100);
    println("I am thinking of a number between 1 and 100.");
    let mut guess: i64 = 0;
    let mut tries: i64 = 0;
    while guess != secret {
        print("Your guess: ");
        let line = read_line();
        guess = line.parse_i64();
        tries = tries + 1;
        if guess < secret {
            println("Too low!");
        } else {
            if guess > secret {
                println("Too high!");
            }
        }
    }
    println("Correct! It took {} tries.", tries);
    return 0;
}