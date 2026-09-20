include "std/ini.cp";

extern fn printf(fmt: *const char, ...) -> i32;
extern fn strlen(s: *const char) -> usize;

fn main() -> i32 {
    let text = "[server]\nhost = 127.0.0.1\nport = 8080\nmax_conn = 128\n\n[logging]\nlevel = info\n; a comment\nfile = app.log\n\ntop = value";
    let cfg = Ini::parse(text);
    if !cfg.valid {
        return 1;
    }
    if !cfg.has("server", "host") {
        return 2;
    }
    printf("host: %s\n", cfg.get("server", "host"));
    printf("port: %s\n", cfg.get("server", "port"));
    printf("log level: %s\n", cfg.get("logging", "level"));
    printf("log file: %s\n", cfg.get("logging", "file"));
    if cfg.has("logging", "host") {
        return 3;
    }
    if cfg.get("missing", "key") != null {
        return 4;
    }
    let sects = cfg.section_names();
    let mut i: usize = 0;
    printf("sections: ");
    while i < sects.len() {
        if i > 0 {
            printf(", ");
        }
        printf("%s", sects.get(i));
        i = i + 1;
    }
    printf("\n");
    sects.free_buf();

    let keys = cfg.key_names("server");
    i = 0;
    printf("server keys: ");
    while i < keys.len() {
        if i > 0 {
            printf(", ");
        }
        printf("%s", keys.get(i));
        i = i + 1;
    }
    printf("\n");
    keys.free_buf();

    let mut cfg2 = Ini::parse("[db]\nengine = sqlite\n");
    if !cfg2.valid {
        return 5;
    }
    printf("db engine: %s\n", cfg2.get("db", "engine"));
    cfg2.set("db", "engine", "postgres");
    printf("db engine after set: %s\n", cfg2.get("db", "engine"));
    if cfg2.has("db", "port") {
        return 6;
    }
    cfg2.set("db", "port", "5432");
    if !cfg2.has("db", "port") {
        return 7;
    }
    printf("db port: %s\n", cfg2.get("db", "port"));
    if cfg2.has("top", "value") {
        return 8;
    }

    cfg.free_buf();
    cfg2.free_buf();
    return 0;
}