use crate::SavedCapture;
use std::collections::VecDeque;

pub const DEFAULT_PREVIEW_LIMIT: usize = 5;

#[derive(Debug, Clone)]
pub struct PreviewStack {
    items: VecDeque<SavedCapture>,
    limit: usize,
}

impl Default for PreviewStack {
    fn default() -> Self {
        Self::new(DEFAULT_PREVIEW_LIMIT)
    }
}

impl PreviewStack {
    pub fn new(limit: usize) -> Self {
        assert!(limit > 0, "preview stack limit must be greater than zero");
        Self {
            items: VecDeque::with_capacity(limit),
            limit,
        }
    }

    pub fn push(&mut self, capture: SavedCapture) {
        self.items.push_front(capture);
        self.items.truncate(self.limit);
    }

    pub fn remove(&mut self, index: usize) -> Option<SavedCapture> {
        self.items.remove(index)
    }

    pub fn newest(&self) -> Option<&SavedCapture> {
        self.items.front()
    }

    pub fn items(&self) -> impl ExactSizeIterator<Item = &SavedCapture> {
        self.items.iter()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn capture(name: &str) -> SavedCapture {
        SavedCapture {
            path: PathBuf::from(name),
            width: 100,
            height: 50,
        }
    }

    #[test]
    fn newest_capture_is_first() {
        let mut stack = PreviewStack::new(3);
        stack.push(capture("one.png"));
        stack.push(capture("two.png"));

        assert_eq!(stack.newest().unwrap().path, PathBuf::from("two.png"));
    }

    #[test]
    fn stack_is_bounded() {
        let mut stack = PreviewStack::new(2);
        stack.push(capture("one.png"));
        stack.push(capture("two.png"));
        stack.push(capture("three.png"));

        let names = stack
            .items()
            .map(|item| item.path.clone())
            .collect::<Vec<_>>();
        assert_eq!(names, vec![PathBuf::from("three.png"), PathBuf::from("two.png")]);
    }
}
