use std::cmp::Ordering;
use std::ops::Range;
pub fn first_and_last_percent<T>(slice: &mut [T], first_percent: i32, last_percent: i32) -> Range<T>
where
    T: PartialOrd + Copy,
{
    let n = slice.len();
    let first_pct_idx = ((first_percent as f32) * 0.01 * (n as f32)) as usize;
    let last_pct_idx = ((last_percent as f32) * 0.01 * (n as f32)) as usize;

    let min_val = {
        let (_, min_val, _) = slice.select_nth_unstable_by(first_pct_idx, |a, b| {
            a.partial_cmp(b).unwrap_or(Ordering::Greater)
        });
        *min_val
    };
    let max_val = {
        let (_, max_val, _) = slice.select_nth_unstable_by(last_pct_idx, |a, b| {
            a.partial_cmp(b).unwrap_or(Ordering::Greater)
        });
        *max_val
    };

    min_val..max_val
}
