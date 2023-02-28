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

pub unsafe fn transmute_boxed_slice<I, O>(s: Box<[I]>) -> Box<[O]> {
    let len = s.len();
    let in_slice_ptr = Box::into_raw(s);

    let new_len = len * std::mem::size_of::<I>() / std::mem::size_of::<O>();
    let out_slice_ptr = std::slice::from_raw_parts_mut(in_slice_ptr as *mut O, new_len);

    Box::from_raw(out_slice_ptr)
}

pub unsafe fn transmute_vec_to_u8<I>(mut s: Vec<I>) -> Vec<u8> {
    s.set_len(std::mem::size_of_val(&s[..]));
    std::mem::transmute(s)
}

/// Select the kth smallest element in a slice
/// 
/// This is a basic implementation of quickselect algorithm: https://fr.wikipedia.org/wiki/Quickselect
/// Some features:
/// * The pivot is chosen randomly between l and r
/// * This does a partial sort of `v`
/// * It performs in O(n) in mean time
/// 
/// # Params
/// * `v` - the slice of values from which the kth smallest element will be found
/// * `l` - the first index of the slice for which the algorithm is applied
/// * `r` - the last index of the slice (inclusive) for which the algorithm is applied
/// * `k` - the index number to find
use rand::Rng;
pub fn select_kth_smallest<T: PartialOrd + Copy>(v: &mut [T], mut l: usize, mut r: usize, k: usize) -> T {
    let mut rng = rand::thread_rng();
    while l < r {
        let pivot = rng.gen_range(l..=r);
        let pivot = partition(v, l, r, pivot);

        if k == pivot {
            return v[k];
        } else if k < pivot {
            r = pivot - 1;
        } else {
            l = pivot + 1;
        }
    }

    v[l]
}

fn partition<T: PartialOrd + Copy>(v: &mut [T], l: usize, r: usize, pivot: usize) -> usize {
    v.swap(pivot, r);
    let pivot = v[r];
    let mut j = l;
    for i in l..r {
        if v[i] < pivot {
            v.swap(i, j);
            j += 1;
        }
    }

    // swap pivot value to values[j]
    v.swap(r, j);
    j
}

mod tests {
    #[test]
    fn test_select_kth_smallest() {
        assert_eq!(super::select_kth_smallest(&mut [2, 4, 5, 9, -1, 5], 0, 5, 2), 4);
        assert_eq!(super::select_kth_smallest(&mut [2], 0, 0, 0), 2);
        assert_eq!(super::select_kth_smallest(&mut [2, 4, 5, 9, -1, 5], 0, 5, 3), 5);
        assert_eq!(super::select_kth_smallest(&mut [2, 4, 5, 9, -1, 5], 0, 5, 4), 5);
        assert_eq!(super::select_kth_smallest(&mut [2, 4, 5, 9, -1, 5], 0, 5, 5), 9);

        assert_eq!(super::select_kth_smallest(&mut [0, 1, 2, 9, 11, 12], 0, 5, 5), 12);

    }
}