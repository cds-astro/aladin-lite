use std::cmp::Ordering;
use std::ops::Range;

#[allow(unused_macros)]
macro_rules! assert_delta {
    ($x:expr, $y:expr, $d:expr) => {
        if ($x - $y).abs() >= $d {
            panic!();
        }
    };
}

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

// Transmute utils functions
#[allow(dead_code)]
pub unsafe fn transmute_boxed_slice<I, O>(s: Box<[I]>) -> Box<[O]> {
    let len = s.len();
    let in_slice_ptr = Box::into_raw(s);

    let new_len = len * std::mem::size_of::<I>() / std::mem::size_of::<O>();
    let out_slice_ptr = std::slice::from_raw_parts_mut(in_slice_ptr as *mut O, new_len);

    Box::from_raw(out_slice_ptr)
}

#[allow(dead_code)]
pub unsafe fn transmute_vec_to_u8<I>(mut s: Vec<I>) -> Vec<u8> {
    s.set_len(std::mem::size_of_val(&s[..]));
    std::mem::transmute(s)
}

pub unsafe fn transmute_vec<I, O>(mut s: Vec<I>) -> Result<Vec<O>, &'static str> {
    if std::mem::size_of::<I>() % std::mem::size_of::<O>() > 0 {
        Err("The input type is not a multiple of the output type")
    } else {
        s.set_len(s.len() * (std::mem::size_of::<I>() / std::mem::size_of::<O>()));
        Ok(std::mem::transmute(s))
    }
}

#[allow(unused)]
pub(super) fn merge_overlapping_intervals(mut intervals: Vec<Range<usize>>) -> Vec<Range<usize>> {
    intervals.sort_unstable_by(|a, b| {
        let cmp = a.start.cmp(&b.start);
        if let Ordering::Equal = cmp {
            a.end.cmp(&b.end)
        } else {
            cmp
        }
    });

    // Merge overlapping intervals in place
    let mut j = 0;

    for i in 1..intervals.len() {
        // If this is not first Interval and overlaps
        // with the previous one
        if intervals[j].end >= intervals[i].start {
            // Merge previous and current Intervals
            intervals[j].end = intervals[j].end.max(intervals[i].end);
        } else {
            j += 1;
            intervals[j] = intervals[i].clone();
        }
    }
    // truncate the indices
    intervals.truncate(j + 1);

    intervals
}

/*
Execute a closure after some delay. This mimics the javascript built-in setTimeout procedure.
*/
#[cfg(target_arch = "wasm32")]
pub(crate) fn set_timeout<F>(f: F, delay: i32)
where
    F: 'static + FnOnce() -> (),
{
    use std::cell::Cell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    let timeout_id = Rc::new(Cell::new(0));
    let t_id = timeout_id.clone();
    let cb = Closure::once_into_js(move || {
        f();

        web_sys::window()
            .unwrap()
            .clear_timeout_with_handle(t_id.get());
    });

    let window = web_sys::window().unwrap();
    timeout_id.set(
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                // Note this method call, which uses `as_ref()` to get a `JsValue`
                // from our `Closure` which is then converted to a `&Function`
                // using the `JsCast::unchecked_ref` function.
                cb.unchecked_ref(),
                delay,
            )
            .unwrap(),
    );
}
