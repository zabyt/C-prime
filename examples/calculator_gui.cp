include "std/raylib.cp";
include "std/ctypes.cp";






fn ftos(v: f64, buf: *mut char) -> i32 {
    let mut x = v;
    let mut i = 0;
    if x < 0.0 {
        buf[0] = '-';
        i = 1;
        x = -x;
    }
    let mut ip: f64 = 0.0;
    let mut fp = modf(x, &mut ip);
    let mut n = ip as u64;
    if n == 0 {
        buf[i] = '0';
        i = i + 1;
    }
    while n > 0 {
        let d = (n % 10) as i32;
        n = n / 10;
        buf[i] = ('0' as i32 + d) as char;
        i = i + 1;
    }
    let mut l = 0;
    let mut r = i - 1;
    while l < r {
        let t = buf[l];
        buf[l] = buf[r];
        buf[r] = t;
        l = l + 1;
        r = r - 1;
    }
    if fp > 0.0 {
        let mut frac = floor(fp * 100000000.0 + 0.5) as u64;
        if frac > 0 {
            buf[i] = '.';
            i = i + 1;
            let start = i;
            let mut w = 0;
            while w < 8 {
                buf[i] = (48 + (frac % 10) as i32) as char;
                i = i + 1;
                frac = frac / 10;
                w = w + 1;
            }
            let mut l = start;
            let mut r = i - 1;
            while l < r {
                let t = buf[l];
                buf[l] = buf[r];
                buf[r] = t;
                l = l + 1;
                r = r - 1;
            }
            while i > start + 1 && buf[i - 1] == '0' {
                i = i - 1;
            }
        }
    }
    buf[i] = 0 as char;
    return i;
}


fn apply(a: f64, b: f64, op: i32) -> f64 {
    if op == 1 {
        return a + b;
    }
    if op == 2 {
        return a - b;
    }
    if op == 3 {
        return a * b;
    }
    if op == 4 {
        if b == 0.0 {
            return 0.0;
        }
        return a / b;
    }
    return b;
}


fn bad_result(a: f64, b: f64, op: i32) -> bool {
    if op == 4 && b == 0.0 {
        return true;
    }
    return false;
}


fn digit_pressed(d: i32, current: *mut f64, fresh: *mut bool, frac: *mut bool, scale: *mut f64, err: *mut bool) {
    if *err {
        *current = 0.0;
        *err = false;
        *fresh = true;
        *frac = false;
        *scale = 1.0;
    }
    if *fresh {
        *current = d as f64;
        *fresh = false;
        *frac = false;
        *scale = 1.0;
    } else if *frac {
        *scale = *scale * 10.0;
        *current = *current + (d as f64) / *scale;
    } else {
        *current = *current * 10.0 + (d as f64);
    }
}

fn decimal_pressed(current: *mut f64, fresh: *mut bool, frac: *mut bool) {
    if *fresh {
        *current = 0.0;
        *fresh = false;
    }
    *frac = true;
}

fn op_pressed(o: i32, stored: *mut f64, current: *mut f64, op: *mut i32, fresh: *mut bool, frac: *mut bool, scale: *mut f64, err: *mut bool) {
    *stored = apply(*stored, *current, *op);
    *err = bad_result(*stored, *current, *op);
    *op = o;
    *fresh = true;
    *frac = false;
    *scale = 1.0;
}

fn equals_pressed(stored: *mut f64, current: *mut f64, op: *mut i32, fresh: *mut bool, frac: *mut bool, scale: *mut f64, err: *mut bool) {
    *current = apply(*stored, *current, *op);
    *err = bad_result(*stored, *current, *op);
    *op = 0;
    *fresh = true;
    *frac = false;
    *scale = 1.0;
}

fn clear_all(stored: *mut f64, current: *mut f64, op: *mut i32, fresh: *mut bool, frac: *mut bool, scale: *mut f64, err: *mut bool) {
    *stored = 0.0;
    *current = 0.0;
    *op = 0;
    *fresh = true;
    *frac = false;
    *scale = 1.0;
    *err = false;
}


fn digit_for(row: i32, col: i32) -> i32 {
    if row == 4 {
        return 0;
    }
    return 7 - (row - 1) * 3 + col;
}


fn button_label(row: i32, col: i32) -> *const char {
    if row == 0 && col == 0 {
        return "C";
    }
    if row == 0 && col == 1 {
        return "+/-";
    }
    if row == 0 && col == 2 {
        return "%";
    }
    if row == 0 {
        return "/";
    }
    if row == 1 && col == 0 {
        return "7";
    }
    if row == 1 && col == 1 {
        return "8";
    }
    if row == 1 && col == 2 {
        return "9";
    }
    if row == 1 {
        return "*";
    }
    if row == 2 && col == 0 {
        return "4";
    }
    if row == 2 && col == 1 {
        return "5";
    }
    if row == 2 && col == 2 {
        return "6";
    }
    if row == 2 {
        return "-";
    }
    if row == 3 && col == 0 {
        return "1";
    }
    if row == 3 && col == 1 {
        return "2";
    }
    if row == 3 && col == 2 {
        return "3";
    }
    if row == 3 {
        return "+";
    }
    if row == 4 && col == 0 {
        return "0";
    }
    if row == 4 && col == 1 {
        return ".";
    }
    if row == 4 && col == 2 {
        return "=";
    }
    return "";
}

fn main() {
    InitWindow(420, 640, "C-Prime Calculator");
    SetTargetFPS(60);

    let mut stored = 0.0;
    let mut current = 0.0;
    let mut op = 0;
    let mut fresh = true;
    let mut frac = false;
    let mut scale = 1.0;
    let mut err = false;
    let buffer: *mut char = malloc(40) as *mut char;

    let bw: i32 = 85;
    let bh: i32 = 70;
    let x0: i32 = 20;
    let y0: i32 = 140;

    while !WindowShouldClose() {
        let mut clicked = false;
        let mut row = 0;
        let mut col = 0;
        if IsMouseButtonPressed(MOUSE_BUTTON_LEFT) {
            let mp = GetMousePosition();
            if mp.x >= (x0 as f32) && mp.y >= (y0 as f32) {
                let cx = (mp.x - (x0 as f32)) / (bw as f32);
                let cy = (mp.y - (y0 as f32)) / (bh as f32);
                if cx >= 0.0 && cx < 4.0 && cy >= 0.0 && cy < 5.0 {
                    clicked = true;
                    col = cx as i32;
                    row = cy as i32;
                }
            }
        }

        let mut key = GetCharPressed();
        while key != 0 {
            if key >= 48 && key <= 57 {
                digit_pressed(key - 48, &mut current, &mut fresh, &mut frac, &mut scale, &mut err);
            } else if key == 46 {
                decimal_pressed(&mut current, &mut fresh, &mut frac);
            } else if key == 43 {
                op_pressed(1, &mut stored, &mut current, &mut op, &mut fresh, &mut frac, &mut scale, &mut err);
            } else if key == 45 {
                op_pressed(2, &mut stored, &mut current, &mut op, &mut fresh, &mut frac, &mut scale, &mut err);
            } else if key == 42 {
                op_pressed(3, &mut stored, &mut current, &mut op, &mut fresh, &mut frac, &mut scale, &mut err);
            } else if key == 47 {
                op_pressed(4, &mut stored, &mut current, &mut op, &mut fresh, &mut frac, &mut scale, &mut err);
            } else if key == 61 {
                equals_pressed(&mut stored, &mut current, &mut op, &mut fresh, &mut frac, &mut scale, &mut err);
            } else if key == 67 || key == 99 {
                clear_all(&mut stored, &mut current, &mut op, &mut fresh, &mut frac, &mut scale, &mut err);
            }
            key = GetCharPressed();
        }

        if clicked {
            let c = row * 4 + col;
            if c == 0 {
                clear_all(&mut stored, &mut current, &mut op, &mut fresh, &mut frac, &mut scale, &mut err);
            } else if c == 1 {
                current = -current;
            } else if c == 2 {
                current = current / 100.0;
            } else if c == 3 {
                op_pressed(4, &mut stored, &mut current, &mut op, &mut fresh, &mut frac, &mut scale, &mut err);
            } else if c == 7 {
                op_pressed(3, &mut stored, &mut current, &mut op, &mut fresh, &mut frac, &mut scale, &mut err);
            } else if c == 11 {
                op_pressed(2, &mut stored, &mut current, &mut op, &mut fresh, &mut frac, &mut scale, &mut err);
            } else if c == 15 {
                op_pressed(1, &mut stored, &mut current, &mut op, &mut fresh, &mut frac, &mut scale, &mut err);
            } else if c == 18 {
                equals_pressed(&mut stored, &mut current, &mut op, &mut fresh, &mut frac, &mut scale, &mut err);
            } else if c == 17 {
                decimal_pressed(&mut current, &mut fresh, &mut frac);
            } else if c == 19 {
            } else {
                digit_pressed(digit_for(row, col), &mut current, &mut fresh, &mut frac, &mut scale, &mut err);
            }
        }

        BeginDrawing();
        ClearBackground(DARKGRAY);

        
        DrawRectangle(20, 20, 380, 100, RAYWHITE);
        DrawRectangleLines(20, 20, 380, 100, BLACK);
        if err {
            DrawText("error", 320, 55, 28, RED);
        } else {
            let n = ftos(current, buffer);
            DrawText(buffer as *const char, 360 - n * 18, 55, 28, BLACK);
        }

        
        let mut i = 0;
        while i < 20 {
            let r = i / 4;
            let c = i % 4;
            let x = x0 + c * bw + 10;
            let y = y0 + r * bh + 10;
            let active = clicked && row == r && col == c;
            if active {
                DrawRectangle(x, y, bw, bh, GOLD);
            } else {
                DrawRectangle(x, y, bw, bh, LIGHTGRAY);
            }
            DrawRectangleLines(x, y, bw, bh, DARKGRAY);
            let label = button_label(r, c);
            if label != "" {
                let tw = MeasureText(label, 26);
                DrawText(label, x + (bw - tw) / 2, y + 20, 26, BLACK);
            }
            i = i + 1;
        }

        DrawFPS(10, 10);
        EndDrawing();
    }

    CloseWindow();
}
