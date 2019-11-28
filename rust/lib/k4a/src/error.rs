#[derive(Debug)]
pub struct Mismatch<T> {
    expected: T,
    actual: T,
}

impl<T> Mismatch<T> {
    pub fn new(expected: T, actual: T) -> Self {
        Self {
            expected: expected,
            actual: actual,
        }
    }
}