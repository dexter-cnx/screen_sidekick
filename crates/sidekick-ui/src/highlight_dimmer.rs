use sidekick_core::{Annotation, DimmerStyle, Point};

pub const DEFAULT_DIMMER_COLOR: &str = "#000000";
pub const DEFAULT_DIMMER_OPACITY: f32 = 0.55;
pub const MIN_HIGHLIGHT_SIZE: f32 = 2.0;

#[derive(Debug, Clone, PartialEq)]
pub struct HighlightDimmerDraft {
    start: Point,
    end: Point,
    style: DimmerStyle,
}

impl HighlightDimmerDraft {
    pub fn new(start: Point, color: impl Into<String>, opacity: f32) -> Self {
        Self {
            start,
            end: start,
            style: DimmerStyle::new(color, opacity),
        }
    }

    pub fn with_defaults(start: Point) -> Self {
        Self::new(start, DEFAULT_DIMMER_COLOR, DEFAULT_DIMMER_OPACITY)
    }

    pub fn start(&self) -> Point {
        self.start
    }

    pub fn end(&self) -> Point {
        self.end
    }

    pub fn style(&self) -> &DimmerStyle {
        &self.style
    }

    pub fn update(&mut self, end: Point) {
        self.end = end;
    }

    pub fn normalized_bounds(&self) -> (f32, f32, f32, f32) {
        let left = self.start.x.min(self.end.x);
        let top = self.start.y.min(self.end.y);
        let width = (self.end.x - self.start.x).abs();
        let height = (self.end.y - self.start.y).abs();
        (left, top, width, height)
    }

    pub fn is_committable(&self) -> bool {
        let (_, _, width, height) = self.normalized_bounds();
        width >= MIN_HIGHLIGHT_SIZE && height >= MIN_HIGHLIGHT_SIZE
    }

    pub fn preview_annotation(&self) -> Annotation {
        let (x, y, w, h) = self.normalized_bounds();
        Annotation::HighlightDimmer {
            x,
            y,
            w,
            h,
            style: self.style.clone(),
        }
    }

    pub fn commit(self) -> Option<Annotation> {
        if !self.is_committable() {
            return None;
        }
        Some(self.preview_annotation())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_draft_commits_platform_neutral_annotation() {
        let mut draft = HighlightDimmerDraft::with_defaults(Point { x: 10.0, y: 20.0 });
        draft.update(Point { x: 40.0, y: 60.0 });

        assert_eq!(
            draft.commit(),
            Some(Annotation::HighlightDimmer {
                x: 10.0,
                y: 20.0,
                w: 30.0,
                h: 40.0,
                style: DimmerStyle::new(DEFAULT_DIMMER_COLOR, DEFAULT_DIMMER_OPACITY),
            })
        );
    }

    #[test]
    fn reversed_drag_normalizes_geometry() {
        let mut draft = HighlightDimmerDraft::with_defaults(Point { x: 40.0, y: 60.0 });
        draft.update(Point { x: 10.0, y: 20.0 });

        assert!(matches!(
            draft.commit(),
            Some(Annotation::HighlightDimmer { x, y, w, h, .. })
                if x == 10.0 && y == 20.0 && w == 30.0 && h == 40.0
        ));
    }

    #[test]
    fn style_normalizes_opacity() {
        let draft = HighlightDimmerDraft::new(Point { x: 0.0, y: 0.0 }, "#112233", 2.0);
        assert_eq!(draft.style().color, "#112233");
        assert_eq!(draft.style().opacity, 1.0);
    }

    #[test]
    fn tiny_draft_does_not_commit() {
        let mut draft = HighlightDimmerDraft::with_defaults(Point { x: 1.0, y: 1.0 });
        draft.update(Point { x: 2.0, y: 20.0 });
        assert!(!draft.is_committable());
        assert_eq!(draft.commit(), None);
    }

    #[test]
    fn preview_annotation_does_not_consume_draft() {
        let mut draft = HighlightDimmerDraft::with_defaults(Point { x: 40.0, y: 60.0 });
        draft.update(Point { x: 10.0, y: 20.0 });

        assert!(draft.is_committable());
        assert!(matches!(
            draft.preview_annotation(),
            Annotation::HighlightDimmer { x, y, w, h, .. }
                if x == 10.0 && y == 20.0 && w == 30.0 && h == 40.0
        ));
        assert_eq!(draft.start(), Point { x: 40.0, y: 60.0 });
        assert_eq!(draft.end(), Point { x: 10.0, y: 20.0 });
    }
}
