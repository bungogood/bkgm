pub const fn comb(n: usize, k: usize) -> usize {
    if k == 0 {
        return 1;
    }
    return (n * comb(n - 1, k - 1)) / k;
}

pub const fn mcomb(n: usize, k: usize) -> usize {
    comb(n + k - 1, k)
}
