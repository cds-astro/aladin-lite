pub fn get_current_time() -> f32 {
    let window = web_sys::window().expect("should have a window in this context");
    let performance = window
        .performance()
        .expect("performance should be available");
    performance.now() as f32
}

pub fn unmortonize(mut x: u64) -> (u32, u32) {
    let mut y = x >> 1;

    x &= 0x5555555555555555;
    x = (x | (x >> 1)) & 0x3333333333333333;
    x = (x | (x >> 2)) & 0x0f0f0f0f0f0f0f0f;
    x = (x | (x >> 4)) & 0x00ff00ff00ff00ff;
    x = (x | (x >> 8)) & 0x0000ffff0000ffff;
    x = (x | (x >> 16)) & 0x00000000ffffffff;

    y &= 0x5555555555555555;
    y = (y | (y >> 1)) & 0x3333333333333333;
    y = (y | (y >> 2)) & 0x0f0f0f0f0f0f0f0f;
    y = (y | (y >> 4)) & 0x00ff00ff00ff00ff;
    y = (y | (y >> 8)) & 0x0000ffff0000ffff;
    y = (y | (y >> 16)) & 0x00000000ffffffff;

    (x as u32, y as u32)
}

pub unsafe fn transmute_boxed_slice<I, O>(s: Box<[I]>) -> Box<[O]> {
    let len = s.len();
    let in_slice_ptr = Box::into_raw(s);

    let new_len = len * std::mem::size_of::<I>() / std::mem::size_of::<O>();
    let out_slice_ptr = std::slice::from_raw_parts_mut(in_slice_ptr as *mut O, new_len);
    
    Box::from_raw(out_slice_ptr)
}