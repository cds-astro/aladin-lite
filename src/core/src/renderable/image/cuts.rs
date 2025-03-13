use std::cmp::Ordering;
use std::ops::Range;
pub fn first_and_last_percent<T>(slice: &mut [T], mut first_percent: i32, mut last_percent: i32) -> Range<T>
where
    T: PartialOrd + Copy,
{
    if first_percent > last_percent {
        std::mem::swap(&mut first_percent, &mut last_percent);
    }

    let n = slice.len();
    let i1 = ((first_percent as f32) * 0.01 * (n as f32)) as usize;
    let i2 = ((last_percent as f32) * 0.01 * (n as f32)) as usize;

    let min_val = {
        let (_, min_val, _) = slice.select_nth_unstable_by(i1, |a, b| a.partial_cmp(b).unwrap_or(Ordering::Greater));
        *min_val
    };
    let max_val = {
        let (_, max_val, _) = slice.select_nth_unstable_by(i2, |a, b| a.partial_cmp(b).unwrap_or(Ordering::Greater));
        *max_val
    };

    min_val..max_val
}
