#![allow(dead_code)]
/// Computes the Leveshtein distance between two input strings
///
/// # Arguments
/// * `a` - a string
/// * `b` - a string
///
/// # Returns
/// * the Levenshtein distance between `a` and `b`
///
/// # Examples
/// ```rust
/// use levenshtein_lite::levenshtein_distance;
/// assert!(levenshtein_distance("abc", "abx") == 1);
/// assert!(levenshtein_distance("abc", "axx") == 2);
/// ```
/// Credit: github.com:danmunson/levenshtein_lite
pub fn levenshtein_distance_matrix(a: &str, b: &str) -> i32 {
    use std::cmp::min;
    let (rowstr, colstr) = (a, b);
    let mut prev = (0..rowstr.len() as i32 + 1).collect::<Vec<i32>>();
    let mut current = prev.clone();
    for (uci, cchar) in colstr.chars().enumerate() {
        current[0] = uci as i32 + 1;
        for (uri, rchar) in rowstr.chars().enumerate() {
            let ri = uri + 1;
            let r_insert_d = prev[ri] + 1;
            let r_del_d = current[ri - 1] + 1;
            let r_match_or_sub_d = if rchar == cchar { prev[ri - 1] } else { prev[ri - 1] + 1 };
            current[ri] = min(r_match_or_sub_d, min(r_insert_d, r_del_d));
        }
        // swap current and prev
        // (current, prev) = (prev, current);
        // Maybe quicker?
        std::mem::swap(&mut prev, &mut current);
    }
    // because of the swap,
    // prev is actually the last set of distances
    prev[prev.len() - 1]
}
