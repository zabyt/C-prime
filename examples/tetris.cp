include "std/raylib.cp";
include "std/ctypes.cp";







const CELL: i32 = 30;
const COLS: i32 = 10;
const ROWS: i32 = 20;
const BX: i32 = 20;
const BY: i32 = 40;


fn rot(m: i32) -> i32 {
    let mut out = 0;
    let mut x = 0;
    while x < 4 {
        let mut y = 0;
        while y < 4 {
            if (m & (1 << (x + y * 4))) != 0 {
                let nx = 3 - y;
                let ny = x;
                out = out | (1 << (nx + ny * 4));
            }
            y = y + 1;
        }
        x = x + 1;
    }
    return out;
}


fn piece_mask(k: i32, r: i32) -> i32 {
    let mut base = 0;
    if k == 0 {
        base = 0x00F0; 
    } else if k == 1 {
        base = 0x0033; 
    } else if k == 2 {
        base = 0x0072; 
    } else if k == 3 {
        base = 0x0036; 
    } else if k == 4 {
        base = 0x0063; 
    } else if k == 5 {
        base = 0x0071; 
    } else {
        base = 0x0074; 
    }
    let mut m = base;
    let mut i = 0;
    while i < r {
        m = rot(m);
        i = i + 1;
    }
    return m;
}


fn collides(board: *mut i32, mask: i32, px: i32, py: i32) -> bool {
    let mut x = 0;
    while x < 4 {
        let mut y = 0;
        while y < 4 {
            if (mask & (1 << (x + y * 4))) != 0 {
                let bx = px + x;
                let by = py + y;
                if bx < 0 || bx >= COLS || by >= ROWS {
                    return true;
                }
                if by >= 0 && board[bx + by * COLS] != 0 {
                    return true;
                }
            }
            y = y + 1;
        }
        x = x + 1;
    }
    return false;
}


fn lock_piece(board: *mut i32, k: i32, mask: i32, px: i32, py: i32) {
    let mut x = 0;
    while x < 4 {
        let mut y = 0;
        while y < 4 {
            if (mask & (1 << (x + y * 4))) != 0 {
                board[(px + x) + (py + y) * COLS] = k + 1;
            }
            y = y + 1;
        }
        x = x + 1;
    }
}


fn clear_lines(board: *mut i32) -> i32 {
    let mut cleared = 0;
    let mut y = ROWS - 1;
    while y >= 0 {
        let mut full = true;
        let mut x = 0;
        while x < COLS {
            if board[x + y * COLS] == 0 {
                full = false;
            }
            x = x + 1;
        }
        if full {
            let mut yy = y;
            while yy > 0 {
                let mut x2 = 0;
                while x2 < COLS {
                    board[x2 + yy * COLS] = board[x2 + (yy - 1) * COLS];
                    x2 = x2 + 1;
                }
                yy = yy - 1;
            }
            let mut x3 = 0;
            while x3 < COLS {
                board[x3] = 0;
                x3 = x3 + 1;
            }
            cleared = cleared + 1;
        } else {
            y = y - 1;
        }
    }
    return cleared;
}


fn piece_color(v: i32) -> Color {
    if v == 1 {
        return SKYBLUE;
    }
    if v == 2 {
        return YELLOW;
    }
    if v == 3 {
        return PURPLE;
    }
    if v == 4 {
        return GREEN;
    }
    if v == 5 {
        return RED;
    }
    if v == 6 {
        return BLUE;
    }
    return ORANGE;
}


fn draw_piece_cells(mask: i32, sx: i32, sy: i32, color: Color) {
    let mut x = 0;
    while x < 4 {
        let mut y = 0;
        while y < 4 {
            if (mask & (1 << (x + y * 4))) != 0 {
                let dx = sx + x * CELL;
                let dy = sy + y * CELL;
                DrawRectangle(dx + 1, dy + 1, CELL - 2, CELL - 2, color);
            }
            y = y + 1;
        }
        x = x + 1;
    }
}


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
        buf[i] = (48 + d) as char;
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
            let mut l2 = start;
            let mut r2 = i - 1;
            while l2 < r2 {
                let t2 = buf[l2];
                buf[l2] = buf[r2];
                buf[r2] = t2;
                l2 = l2 + 1;
                r2 = r2 - 1;
            }
            while i > start + 1 && buf[i - 1] == '0' {
                i = i - 1;
            }
        }
    }
    buf[i] = 0 as char;
    return i;
}

fn main() {
    InitWindow(460, 680, "C-Prime Tetris");
    SetTargetFPS(60);
    SetRandomSeed(12345);

    let board: *mut i32 = malloc((COLS * ROWS * 4) as usize) as *mut i32;
    let numbuf: *mut char = malloc(16) as *mut char;

    let mut k = 0;
    let mut r = 0;
    let mut mask = piece_mask(0, 0);
    let mut px = 3;
    let mut py = 0;
    let mut next_k = 0;
    let mut score = 0;
    let mut lines = 0;
    let mut level = 1;
    let mut last: f64 = 0.0;
    let mut paused = false;
    let mut over = false;
    let mut restarted = true;

    while !WindowShouldClose() {
        let mut spawn = false;

        if over {
            BeginDrawing();
            ClearBackground(BLACK);
            DrawText("GAME OVER", 110, 280, 40, RED);
            DrawText("press ENTER to restart", 120, 340, 20, LIGHTGRAY);
            EndDrawing();
            if IsKeyPressed(KEY_ENTER) {
                over = false;
                restarted = true;
            }
        } else {
            if IsKeyPressed(80) {
                paused = !paused;
            }

            if !paused {
                if IsKeyPressed(KEY_LEFT) {
                    if !collides(board, mask, px - 1, py) {
                        px = px - 1;
                    }
                }
                if IsKeyPressed(KEY_RIGHT) {
                    if !collides(board, mask, px + 1, py) {
                        px = px + 1;
                    }
                }
                if IsKeyPressed(KEY_UP) {
                    let nm = piece_mask(k, (r + 1) % 4);
                    if !collides(board, nm, px, py) {
                        r = (r + 1) % 4;
                        mask = nm;
                    }
                }
                if IsKeyDown(KEY_DOWN) {
                    if !collides(board, mask, px, py + 1) {
                        py = py + 1;
                        score = score + 1;
                        last = GetTime();
                    }
                }
                if IsKeyPressed(KEY_SPACE) {
                    while !collides(board, mask, px, py + 1) {
                        py = py + 1;
                        score = score + 2;
                    }
                    spawn = true;
                }

                let now = GetTime();
                let speed = 0.8 - (level as f64) * 0.05;
                if now - last > speed {
                    if !collides(board, mask, px, py + 1) {
                        py = py + 1;
                    } else {
                        spawn = true;
                    }
                    last = now;
                }

                if spawn {
                    lock_piece(board, k, mask, px, py);
                    let cleared = clear_lines(board);
                    lines = lines + cleared;
                    if cleared == 1 {
                        score = score + 100;
                    } else if cleared == 2 {
                        score = score + 300;
                    } else if cleared == 3 {
                        score = score + 500;
                    } else if cleared == 4 {
                        score = score + 800;
                    }
                    level = lines / 10 + 1;
                    k = next_k;
                    next_k = GetRandomValue(0, 6);
                    r = 0;
                    mask = piece_mask(k, 0);
                    px = 3;
                    py = 0;
                    if collides(board, mask, px, py) {
                        over = true;
                    }
                    last = now;
                }
            }

            BeginDrawing();
            ClearBackground(BLACK);

            
            DrawRectangleLines(BX - 2, BY - 2, COLS * CELL + 4, ROWS * CELL + 4, DARKGRAY);

            
            let mut y = 0;
            while y < ROWS {
                let mut x = 0;
                while x < COLS {
                    let v = board[x + y * COLS];
                    if v != 0 {
                        let color = piece_color(v);
                        DrawRectangle(BX + x * CELL + 1, BY + y * CELL + 1, CELL - 2, CELL - 2, color);
                    } else {
                        DrawRectangle(BX + x * CELL + 1, BY + y * CELL + 1, CELL - 2, CELL - 2, DARKGRAY);
                    }
                    x = x + 1;
                }
                y = y + 1;
            }

            
            draw_piece_cells(mask, BX + px * CELL, BY + py * CELL, piece_color(k + 1));

            
            DrawText("SCORE", 340, 60, 20, LIGHTGRAY);
            let n = ftos(score as f64, numbuf);
            DrawText(numbuf as *const char, 340, 90, 24, WHITE);
            DrawText("LINES", 340, 140, 20, LIGHTGRAY);
            let n2 = ftos(lines as f64, numbuf);
            DrawText(numbuf as *const char, 340, 170, 24, WHITE);
            DrawText("LEVEL", 340, 220, 20, LIGHTGRAY);
            let n3 = ftos(level as f64, numbuf);
            DrawText(numbuf as *const char, 340, 250, 24, WHITE);
            DrawText("NEXT", 340, 320, 20, LIGHTGRAY);
            let nmask = piece_mask(next_k, 0);
            draw_piece_cells(nmask, 340, 350, piece_color(next_k + 1));

            if paused {
                DrawText("PAUSED", 100, 300, 40, GOLD);
            }

            DrawFPS(10, 10);
            EndDrawing();
        }

        if restarted {
            restarted = false;
            score = 0;
            lines = 0;
            level = 1;
            let mut i = 0;
            while i < COLS * ROWS {
                board[i] = 0;
                i = i + 1;
            }
            k = GetRandomValue(0, 6);
            next_k = GetRandomValue(0, 6);
            r = 0;
            mask = piece_mask(k, 0);
            px = 3;
            py = 0;
            last = GetTime();
            paused = false;
        }
    }

    CloseWindow();
}
