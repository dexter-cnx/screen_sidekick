use sidekick_core::{Annotation, Point, TextStyle};

pub const DEFAULT_TEXT_COLOR: &str = "#ffffffff";
pub const DEFAULT_TEXT_FONT_SIZE: f32 = 18.0;

#[derive(Debug, Clone, PartialEq)]
pub struct TextAnnotationDraft {
    position: Point,
}

impl TextAnnotationDraft {
    pub fn new(position: Point) -> Self {
        Self { position }
    }

    pub fn position(&self) -> Point {
        self.position
    }

    pub fn commit(&self, text: impl Into<String>) -> Option<Annotation> {
        let text = text.into();
        if text.trim().is_empty() {
            return None;
        }

        Some(Annotation::Text {
            x: self.position.x,
            y: self.position.y,
            text,
            style: default_text_style(),
        })
    }
}

pub fn default_text_style() -> TextStyle {
    TextStyle {
        color: DEFAULT_TEXT_COLOR.to_owned(),
        font_size: DEFAULT_TEXT_FONT_SIZE,
        background: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commits_non_empty_text_at_selected_position() {
        let draft = TextAnnotationDraft::new(Point { x: 12.0, y: 34.0 });
        let annotation = draft.commit("สวัสดี 😀").expect("text should commit");

        assert_eq!(
            annotation,
            Annotation::Text {
                x: 12.0,
                y: 34.0,
                text: "สวัสดี 😀".to_owned(),
                style: default_text_style(),
            }
        );
    }

    #[test]
    fn ignores_empty_or_whitespace_only_text() {
        let draft = TextAnnotationDraft::new(Point { x: 1.0, y: 2.0 });

        assert_eq!(draft.commit(""), None);
        assert_eq!(draft.commit("   \n\t"), None);
    }
}
