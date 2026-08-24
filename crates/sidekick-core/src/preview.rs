use crate::SavedCapture;
use std::collections::VecDeque;

pub const DEFAULT_PREVIEW_LIMIT: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewVisibility {
    Hidden,
    Expanded,
    Peek,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreviewVisibilityState {
    visibility: PreviewVisibility,
}

impl Default for PreviewVisibilityState {
    fn default() -> Self {
        Self {
            visibility: PreviewVisibility::Hidden,
        }
    }
}

impl PreviewVisibilityState {
    pub fn visibility(&self) -> PreviewVisibility {
        self.visibility
    }

    pub fn on_capture(&mut self) {
        self.visibility = PreviewVisibility::Expanded;
    }

    pub fn on_auto_dismiss(&mut self) {
        if self.visibility == PreviewVisibility::Expanded {
            self.visibility = PreviewVisibility::Peek;
        }
    }

    pub fn on_peek_activated(&mut self) {
        if self.visibility == PreviewVisibility::Peek {
            self.visibility = PreviewVisibility::Expanded;
        }
    }

    pub fn on_stack_empty(&mut self) {
        self.visibility = PreviewVisibility::Hidden;
    }
}

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
        assert_eq!(
            names,
            vec![PathBuf::from("three.png"), PathBuf::from("two.png")]
        );
    }

    #[test]
    fn visibility_starts_hidden_and_expands_on_capture() {
        let mut state = PreviewVisibilityState::default();

        assert_eq!(state.visibility(), PreviewVisibility::Hidden);
        state.on_capture();
        assert_eq!(state.visibility(), PreviewVisibility::Expanded);
    }

    #[test]
    fn auto_dismiss_collapses_expanded_preview_to_peek() {
        let mut state = PreviewVisibilityState::default();
        state.on_capture();

        state.on_auto_dismiss();

        assert_eq!(state.visibility(), PreviewVisibility::Peek);
    }

    #[test]
    fn activating_peek_restores_expanded_preview() {
        let mut state = PreviewVisibilityState::default();
        state.on_capture();
        state.on_auto_dismiss();

        state.on_peek_activated();

        assert_eq!(state.visibility(), PreviewVisibility::Expanded);
    }

    #[test]
    fn empty_stack_hides_preview_from_any_visible_state() {
        let mut state = PreviewVisibilityState::default();
        state.on_capture();
        state.on_stack_empty();
        assert_eq!(state.visibility(), PreviewVisibility::Hidden);

        state.on_capture();
        state.on_auto_dismiss();
        state.on_stack_empty();
        assert_eq!(state.visibility(), PreviewVisibility::Hidden);
    }

    #[test]
    fn irrelevant_visibility_events_do_not_create_visible_state() {
        let mut state = PreviewVisibilityState::default();

        state.on_auto_dismiss();
        state.on_peek_activated();

        assert_eq!(state.visibility(), PreviewVisibility::Hidden);
    }
}
