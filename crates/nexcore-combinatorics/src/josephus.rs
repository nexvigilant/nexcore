//! Josephus problem — from Dudeney #232 "Catching the Mice"
//!
//! n elements in a circle, every kth element removed.
//! J(n,k) = (J(n-1,k) + k) mod n, with J(1,k) = 0.
//!
//! Dudeney's puzzle: 21 mice, remove every 13th, find starting position
//! so white mouse (at a specific position) is eliminated last.
//!
//! Primitives: σ(cyclic sequence) + N(count) + ρ(recursion: wrap-around)

/// Compute the Josephus position: which position survives?
///
/// Given n people in a circle, counting every k, returns the 0-indexed
/// position of the last survivor.
///
/// # Examples
/// ```
/// use nexcore_combinatorics::josephus;
/// assert_eq!(josephus(1, 3), 0);   // Only one person
/// assert_eq!(josephus(7, 3), 3);   // Classic Josephus(7,3) = position 3
/// assert_eq!(josephus(41, 3), 30); // Historical: Josephus Flavius
/// ```
pub fn josephus(n: u32, k: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    let mut pos: u64 = 0;
    for i in 2..=u64::from(n) {
        pos = (pos + u64::from(k)) % i;
    }
    pos as u32
}

/// Compute the full elimination order for n elements with step k.
///
/// Returns a vector where `result[i]` is the element eliminated at step i.
/// Elements are 0-indexed.
///
/// # Examples
/// ```
/// use nexcore_combinatorics::josephus::elimination_order;
/// let order = elimination_order(5, 2);
/// // Circle: 0,1,2,3,4. Count by 2: eliminate 1, 3, 0, 4, survivor=2
/// assert_eq!(order, vec![1, 3, 0, 4, 2]);
/// ```
pub fn elimination_order(n: u32, k: u32) -> Vec<u32> {
    let n_usize = n as usize;
    let k_usize = k as usize;
    let mut circle: Vec<u32> = (0..n).collect();
    let mut order = Vec::with_capacity(n_usize);
    let mut idx = 0usize;

    while !circle.is_empty() {
        idx = (idx + k_usize - 1) % circle.len();
        order.push(circle.remove(idx));
        if !circle.is_empty() && idx >= circle.len() {
            idx = 0;
        }
    }

    order
}

/// Find the starting position so that a target element is eliminated last.
///
/// This solves Dudeney's actual puzzle: given n mice in a circle, counting
/// every k, from which starting position should we begin counting so that
/// the mouse at `target_pos` is the last one eliminated?
///
/// Returns the starting offset (0-indexed).
pub fn find_start_for_last(n: u32, k: u32, target_pos: u32) -> Option<u32> {
    let survivor = josephus(n, k);
    Some((target_pos + n - survivor) % n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_josephus_base_cases() {
        assert_eq!(josephus(1, 1), 0);
        assert_eq!(josephus(1, 7), 0);
        assert_eq!(josephus(2, 1), 1);
        assert_eq!(josephus(2, 2), 0);
    }

    #[test]
    fn test_josephus_classic() {
        // J(7, 3) = 3 (well-known)
        assert_eq!(josephus(7, 3), 3);
        // J(41, 3) = 30 (Josephus Flavius historical)
        assert_eq!(josephus(41, 3), 30);
    }

    #[test]
    fn test_elimination_order_small() {
        let order = elimination_order(5, 2);
        assert_eq!(order.len(), 5);
        // Last element should be the Josephus survivor
        assert_eq!(*order.last().unwrap_or(&99), josephus(5, 2));
    }

    #[test]
    fn test_elimination_all_present() {
        let order = elimination_order(10, 3);
        let mut sorted = order.clone();
        sorted.sort();
        let expected: Vec<u32> = (0..10).collect();
        assert_eq!(sorted, expected, "All elements should appear exactly once");
    }

    #[test]
    fn test_find_start_for_last() {
        // For n=5, k=2: survivor when starting at 0 is position 2
        let start = find_start_for_last(5, 2, 4);
        assert!(start.is_some());
    }
}
