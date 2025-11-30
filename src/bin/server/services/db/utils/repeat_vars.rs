pub fn repeat_vars(count: usize) -> String {
    let mut s = "?,".repeat(count);
    s.pop();
    s
}
