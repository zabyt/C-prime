include "std/raylib.cp";

fn main() {
    InitWindow(800, 450, "C-Prime + raylib hello");
    SetTargetFPS(60);
    let frames = 0;
    while !WindowShouldClose() {
        BeginDrawing();
        ClearBackground(RAYWHITE);
        DrawRectangle(100, 100, 200, 100, SKYBLUE);
        DrawRectangleLines(100, 100, 200, 100, DARKBLUE);
        DrawText("Hello from C-Prime!", 140, 130, 24, BLACK);
        DrawText("mouse:", 100, 300, 20, DARKGRAY);
        DrawText("(waiting)", 200, 300, 20, DARKGRAY);
        let x = GetMouseX();
        let y = GetMouseY();
        DrawCircle(x, y, 12.0, RED);
        DrawFPS(10, 10);
        EndDrawing();
    }
    CloseWindow();
}
