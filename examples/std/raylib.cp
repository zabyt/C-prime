






struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

struct Vector2 {
    x: f32,
    y: f32,
}

struct Rectangle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

const LIGHTGRAY: Color = Color { r: 200, g: 200, b: 200, a: 255 };
const GRAY: Color = Color { r: 130, g: 130, b: 130, a: 255 };
const DARKGRAY: Color = Color { r: 80, g: 80, b: 80, a: 255 };
const YELLOW: Color = Color { r: 253, g: 249, b: 0, a: 255 };
const GOLD: Color = Color { r: 255, g: 203, b: 0, a: 255 };
const ORANGE: Color = Color { r: 255, g: 161, b: 0, a: 255 };
const PINK: Color = Color { r: 255, g: 109, b: 194, a: 255 };
const RED: Color = Color { r: 230, g: 41, b: 55, a: 255 };
const MAROON: Color = Color { r: 190, g: 33, b: 55, a: 255 };
const GREEN: Color = Color { r: 0, g: 228, b: 48, a: 255 };
const LIME: Color = Color { r: 0, g: 158, b: 47, a: 255 };
const DARKGREEN: Color = Color { r: 0, g: 117, b: 44, a: 255 };
const SKYBLUE: Color = Color { r: 102, g: 191, b: 255, a: 255 };
const BLUE: Color = Color { r: 0, g: 121, b: 241, a: 255 };
const DARKBLUE: Color = Color { r: 0, g: 82, b: 172, a: 255 };
const PURPLE: Color = Color { r: 200, g: 122, b: 255, a: 255 };
const VIOLET: Color = Color { r: 135, g: 60, b: 190, a: 255 };
const DARKPURPLE: Color = Color { r: 112, g: 31, b: 126, a: 255 };
const BEIGE: Color = Color { r: 211, g: 176, b: 131, a: 255 };
const BROWN: Color = Color { r: 127, g: 106, b: 79, a: 255 };
const DARKBROWN: Color = Color { r: 76, g: 63, b: 47, a: 255 };
const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
const BLANK: Color = Color { r: 0, g: 0, b: 0, a: 0 };
const MAGENTA: Color = Color { r: 255, g: 0, b: 255, a: 255 };
const RAYWHITE: Color = Color { r: 245, g: 245, b: 245, a: 255 };


extern fn InitWindow(width: i32, height: i32, title: *const char) -> void;
extern fn CloseWindow() -> void;
extern fn WindowShouldClose() -> bool;
extern fn SetWindowSize(width: i32, height: i32) -> void;
extern fn GetScreenWidth() -> i32;
extern fn GetScreenHeight() -> i32;


extern fn SetTargetFPS(fps: i32) -> void;
extern fn GetFrameTime() -> f32;
extern fn GetTime() -> f64;


extern fn ClearBackground(color: Color) -> void;
extern fn BeginDrawing() -> void;
extern fn EndDrawing() -> void;
extern fn DrawFPS(posX: i32, posY: i32) -> void;
extern fn DrawText(text: *const char, posX: i32, posY: i32, fontSize: i32, color: Color) -> void;
extern fn MeasureText(text: *const char, fontSize: i32) -> i32;
extern fn DrawLine(startPosX: i32, startPosY: i32, endPosX: i32, endPosY: i32, color: Color) -> void;
extern fn DrawLineEx(startPos: Vector2, endPos: Vector2, thick: f32, color: Color) -> void;
extern fn DrawCircle(centerX: i32, centerY: i32, radius: f32, color: Color) -> void;
extern fn DrawCircleV(center: Vector2, radius: f32, color: Color) -> void;
extern fn DrawCircleLines(centerX: i32, centerY: i32, radius: f32, color: Color) -> void;
extern fn DrawRectangle(posX: i32, posY: i32, width: i32, height: i32, color: Color) -> void;
extern fn DrawRectangleV(position: Vector2, size: Vector2, color: Color) -> void;
extern fn DrawRectangleRec(rec: Rectangle, color: Color) -> void;
extern fn DrawRectangleLines(posX: i32, posY: i32, width: i32, height: i32, color: Color) -> void;
extern fn DrawRectangleLinesEx(rec: Rectangle, lineThick: f32, color: Color) -> void;


extern fn IsKeyPressed(key: i32) -> bool;
extern fn IsKeyDown(key: i32) -> bool;
extern fn IsKeyReleased(key: i32) -> bool;
extern fn IsKeyUp(key: i32) -> bool;
extern fn GetKeyPressed() -> i32;
extern fn GetCharPressed() -> i32;


extern fn IsMouseButtonPressed(button: i32) -> bool;
extern fn IsMouseButtonDown(button: i32) -> bool;
extern fn IsMouseButtonReleased(button: i32) -> bool;
extern fn GetMouseX() -> i32;
extern fn GetMouseY() -> i32;
extern fn GetMousePosition() -> Vector2;
extern fn GetMouseWheelMove() -> f32;


extern fn SetRandomSeed(seed: u32) -> void;
extern fn GetRandomValue(min: i32, max: i32) -> i32;


extern fn Fade(color: Color, alpha: f32) -> Color;
extern fn ColorFromHSV(hue: f32, saturation: f32, value: f32) -> Color;
extern fn GetColor(hexValue: u32) -> Color;


const KEY_SPACE: i32 = 32;
const KEY_ENTER: i32 = 257;
const KEY_ESCAPE: i32 = 256;
const KEY_BACKSPACE: i32 = 259;
const KEY_LEFT: i32 = 263;
const KEY_RIGHT: i32 = 262;
const KEY_UP: i32 = 265;
const KEY_DOWN: i32 = 264;
const KEY_ZERO: i32 = 48;
const KEY_ONE: i32 = 49;
const KEY_TWO: i32 = 50;
const KEY_THREE: i32 = 51;
const KEY_FOUR: i32 = 52;
const KEY_FIVE: i32 = 53;
const KEY_SIX: i32 = 54;
const KEY_SEVEN: i32 = 55;
const KEY_EIGHT: i32 = 56;
const KEY_NINE: i32 = 57;


const MOUSE_BUTTON_LEFT: i32 = 0;
const MOUSE_BUTTON_RIGHT: i32 = 1;
const MOUSE_BUTTON_MIDDLE: i32 = 2;
