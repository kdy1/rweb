pub(crate) fn insert_slash(mut patterns: Vec<String>) -> Vec<String> {
    for path in &mut patterns {
        if !path.is_empty() && !path.starts_with('/') {
            path.insert(0, '/');
        };
    }
    patterns
}
