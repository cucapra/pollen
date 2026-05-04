
pub fn compute_windows(path_len: usize, window_size: usize) -> Vec<(usize, usize)> {
    let num_windows = path_len.div_ceil(window_size);
    let mut windows = Vec::with_capacity(num_windows);
    let mut start = 0;
    while start < path_len {
        let end = (start + window_size).min(path_len);
        windows.push((start, end));
        start = end;
    }
    windows
}
