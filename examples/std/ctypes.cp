

extern fn malloc(size: usize) -> *mut void;
extern fn free(ptr: *mut void) -> void;
extern fn calloc(count: usize, size: usize) -> *mut void;
extern fn realloc(ptr: *mut void, size: usize) -> *mut void;
extern fn strlen(s: *const char) -> usize;
extern fn memcpy(dest: *mut void, src: *const void, count: usize) -> *mut void;
extern fn memset(dest: *mut void, value: i32, count: usize) -> *mut void;
extern fn memcmp(a: *const void, b: *const void, count: usize) -> i32;
extern fn atoi(s: *const char) -> i32;
extern fn atol(s: *const char) -> i64;
extern fn atof(s: *const char) -> f64;
extern fn strcmp(a: *const char, b: *const char) -> i32;
extern fn strcpy(dest: *mut char, src: *const char) -> *mut char;
extern fn toupper(c: char) -> char;
extern fn tolower(c: char) -> char;
extern fn abs(x: i32) -> i32;
extern fn labs(x: i64) -> i64;
extern fn fabs(x: f64) -> f64;
extern fn floor(x: f64) -> f64;
extern fn ceil(x: f64) -> f64;
extern fn round(x: f64) -> f64;
extern fn sqrt(x: f64) -> f64;
extern fn pow(x: f64, y: f64) -> f64;
extern fn fmod(x: f64, y: f64) -> f64;
extern fn modf(x: f64, intpart: *mut f64) -> f64;
extern fn rand() -> i32;
extern fn srand(seed: u32) -> void;
