pub trait RedBlack<T> {
    fn new() -> Self;
    fn insert(&mut self, key: T);
    fn delete(&mut self, key: T) -> Option<T>;
    fn search(&mut self, key: T) -> Option<T>;
}
