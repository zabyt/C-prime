






extern fn fopen(path: *const char, mode: *const char) -> *mut void;
extern fn fclose(f: *mut void) -> i32;
extern fn fread(buf: *mut void, size: usize, count: usize, f: *mut void) -> usize;
extern fn fwrite(buf: *const void, size: usize, count: usize, f: *mut void) -> usize;
extern fn fseek(f: *mut void, offset: i64, origin: i32) -> i32;
extern fn ftell(f: *mut void) -> i64;
extern fn fgets(buf: *mut char, max: i32, f: *mut void) -> *mut char;
extern fn remove(path: *const char) -> i32;
extern fn rename(from: *const char, to: *const char) -> i32;
extern fn malloc(size: usize) -> *mut void;
extern fn free(ptr: *mut void) -> void;

fn open_read(path: *const char) -> *mut void {
    return fopen(path, "rb");
}

fn open_write(path: *const char) -> *mut void {
    return fopen(path, "wb");
}

fn open_append(path: *const char) -> *mut void {
    return fopen(path, "ab");
}

fn open_text_read(path: *const char) -> *mut void {
    return fopen(path, "r");
}

fn open_text_write(path: *const char) -> *mut void {
    return fopen(path, "w");
}

fn close(f: *mut void) {
    fclose(f);
}

fn read_bytes(f: *mut void, buf: *mut void, count: usize) -> usize {
    return fread(buf, 1, count, f);
}

fn write_bytes(f: *mut void, buf: *const void, count: usize) -> usize {
    return fwrite(buf, 1, count, f);
}


fn seek(f: *mut void, offset: i64, origin: i32) {
    fseek(f, offset, origin);
}

fn file_size(f: *mut void) -> i64 {
    let pos: i64 = ftell(f);
    fseek(f, 0, 2);
    let size: i64 = ftell(f);
    fseek(f, pos, 0);
    return size;
}




fn read_whole_file(path: *const char, out_data: *mut *mut char, out_len: *mut usize) -> bool {
    let f = fopen(path, "rb");
    if f == null {
        return false;
    }
    let size: i64 = file_size(f);
    if size < 0 {
        fclose(f);
        return false;
    }
    let n: usize = size as usize;
    let buf = malloc(n + 1) as *mut char;
    if buf == null {
        fclose(f);
        return false;
    }
    let got = fread(buf as *mut void, 1, n, f);
    fclose(f);
    buf[got] = 0 as char;
    out_data[0] = buf;
    out_len[0] = got;
    return true;
}


fn write_whole_file(path: *const char, data: *const char, len: usize) -> bool {
    let f = fopen(path, "wb");
    if f == null {
        return false;
    }
    let wrote = fwrite(data as *const void, 1, len, f);
    fclose(f);
    return wrote == len;
}



fn read_line(f: *mut void, buf: *mut char, max: i32) -> *mut char {
    return fgets(buf, max, f);
}

fn delete_file(path: *const char) -> bool {
    return remove(path) == 0;
}

fn rename_file(from: *const char, to: *const char) -> bool {
    return rename(from, to) == 0;
}