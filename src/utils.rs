pub const fn comb(n: usize, k: usize) -> usize {
    match (n, k) {
        (0, _) => 0,
        (_, 0) => 1,
        (n, k) if n == k => 1,
        (n, k) if n < k => 0,
        _ => comb(n - 1, k - 1) + comb(n - 1, k),
    }
}

pub const fn mcomb(n: usize, k: usize) -> usize {
    comb(n + k - 1, k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comb() {
        assert_eq!(comb(10, 4), 210);
        assert_eq!(comb(52, 5), 2598960);
    }

    #[test]
    fn test_mcomb() {
        assert_eq!(mcomb(7, 15), 54264);
        assert_eq!(mcomb(26, 3), 3276);
        assert_eq!(mcomb(25, 3), 2925);
        assert_eq!(mcomb(24, 3), 2600);
        assert_eq!(mcomb(23, 3), 2300);
    }
}
