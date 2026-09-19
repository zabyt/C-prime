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

