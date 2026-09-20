include "vec.cp";
include "hashmap.cp";

fn ini_skip_ws(s: *const char, i: usize) -> usize {
    let mut i = i;
    while s[i] == ' ' as char || s[i] == '\t' as char {
        i = i + 1;
    }
    return i;
}

fn ini_is_ws(c: char) -> bool {
    if c == ' ' as char {
        return true;
    }
    if c == '\t' as char {
        return true;
    }
    if c == '\r' as char {
        return true;
    }
    return false;
}

fn ini_sub_cstr(s: *const char, start: usize, end: usize) -> *mut char {
    let mut i = start;
    while i < end && ini_is_ws(s[i]) {
        i = i + 1;
    }
    let mut j = end;
    while j > i && ini_is_ws(s[j - 1]) {
        j = j - 1;
    }
    if j <= i {
        let out = malloc(1) as *mut char;
        out[0] = 0 as char;
        return out;
    }
    let n = j - i;
    let out = malloc(n + 1) as *mut char;
    let mut k: usize = 0;
    while k < n {
        out[k] = s[i + k];
        k = k + 1;
    }
    out[n] = 0 as char;
    return out;
}

fn ini_is_comment(c: char) -> bool {
    if c == ';' as char {
        return true;
    }
    if c == '#' as char {
        return true;
    }
    return false;
}

struct Ini {
    sections: HashMapStr[HashMapStr[*mut char]],
    valid: bool,
}

impl Ini {
    fn parse(text: *const char) -> Ini {
        let mut ini: Ini = Ini { sections: HashMapStr::new(), valid: false };
        let mut cur = malloc(1) as *mut char;
        cur[0] = 0 as char;
        let mut i: usize = 0;
        while text[i] != 0 as char {
            let start = i;
            while text[i] != 0 as char && text[i] != '\n' as char {
                i = i + 1;
            }
            let end = i;
            if text[i] == '\n' as char {
                i = i + 1;
            }
            let mut p = ini_skip_ws(text, start);
            if p >= end {
                continue;
            }
            if ini_is_comment(text[p]) {
                continue;
            }
            if text[p] == '[' as char {
                let mut q = p + 1;
                while q < end && text[q] != ']' as char {
                    q = q + 1;
                }
                free(cur as *mut void);
                cur = ini_sub_cstr(text, p + 1, q);
                continue;
            }
            let mut eq = p;
            while eq < end && text[eq] != '=' as char && text[eq] != '\r' as char {
                eq = eq + 1;
            }
            if eq >= end || text[eq] != '=' as char {
                continue;
            }
            let key = ini_sub_cstr(text, p, eq);
            let val = ini_sub_cstr(text, eq + 1, end);
            let sl = ini.sections.find(cur);
            if sl == null {
                let mut sec: HashMapStr[*mut char] = HashMapStr::new();
                sec.put(key, val);
                ini.sections.put(cur, sec);
            } else {
                let mut sec: HashMapStr[*mut char] = sl.value;
                let v = sec.find(key);
                if v != null {
                    free(v.value as *mut void);
                }
                sec.put(key, val);
                sl.value = sec;
            }
            free(key as *mut void);
        }
        free(cur as *mut void);
        ini.valid = true;
        return ini;
    }

    fn get(&self, section: *const char, key: *const char) -> *mut char {
        if !self.valid {
            return null;
        }
        let sl = self.sections.find(section);
        if sl == null {
            return null;
        }
        let sec: HashMapStr[*mut char] = sl.value;
        let v = sec.find(key);
        if v == null {
            return null;
        }
        return v.value;
    }

    fn has(&self, section: *const char, key: *const char) -> bool {
        return self.get(section, key) != null;
    }

    fn set(&mut self, section: *const char, key: *const char, value: *const char) {
        let sl = self.sections.find(section);
        if sl == null {
            let mut sec: HashMapStr[*mut char] = HashMapStr::new();
            sec.put(key, dup_cstr(value));
            self.sections.put(section, sec);
        } else {
            let mut sec: HashMapStr[*mut char] = sl.value;
            let v = sec.find(key);
            if v != null {
                free(v.value as *mut void);
            }
            sec.put(key, dup_cstr(value));
            sl.value = sec;
        }
    }

    fn section_names(&self) -> Vec[*mut char] {
        return self.sections.keys();
    }

    fn key_names(&self, section: *const char) -> Vec[*mut char] {
        let sl = self.sections.find(section);
        if sl == null {
            let mut out: Vec[*mut char] = Vec::new();
            return out;
        }
        let sec: HashMapStr[*mut char] = sl.value;
        return sec.keys();
    }

    fn free_buf(&self) {
        if !self.valid {
            return;
        }
        let mut j: usize = 0;
        while j < self.sections.size {
            let s: SlotStr[HashMapStr[*mut char]] = self.sections.data[j];
            if s.used && !s.deleted {
                let mut sec: HashMapStr[*mut char] = s.value;
                let mut k: usize = 0;
                while k < sec.size {
                    let ss: SlotStr[*mut char] = sec.data[k];
                    if ss.used && !ss.deleted {
                        free(ss.value as *mut void);
                    }
                    k = k + 1;
                }
                sec.free_buf();
            }
            j = j + 1;
        }
        self.sections.free_buf();
    }
}