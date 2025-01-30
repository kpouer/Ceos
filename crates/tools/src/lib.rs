pub mod range;

#[inline]
pub fn contains(line: &str, filter: &str) -> bool {
    #[cfg(not(feature = "simd"))]
    return contains_std(line, filter);
    #[cfg(feature = "simd")]
    contains_simd(line, filter)
}

#[inline]
#[cfg(not(feature = "simd"))]
fn contains_std(line: &str, filter: &str) -> bool {
    line.contains(filter)
}

#[inline]
#[cfg(feature = "simd")]
fn contains_simd(line: &str, filter: &str) -> bool {
    memchr::memmem::find(line.as_bytes(), filter.as_bytes()).is_some()
}

#[inline]
pub fn find(line: &str, filter: &str) -> Option<usize> {
    #[cfg(not(feature = "simd"))]
    return find_std(line, filter);
    #[cfg(feature = "simd")]
    find_simd(line, filter)
}

#[inline]
#[cfg(not(feature = "simd"))]
fn find_std(line: &str, filter: &str) -> Option<usize> {
    line.find(filter)
}

#[inline]
#[cfg(feature = "simd")]
fn find_simd(line: &str, filter: &str) -> Option<usize> {
    memchr::memmem::find(line.as_bytes(), filter.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains() {
        let line = "The plot on the left displays the average time per iteration for this benchmark. The shaded region shows the estimated probability of an iteration taking a";
        assert!(contains(line, "shaded"));
        assert!(!contains(line, "hello"));
    }
}
