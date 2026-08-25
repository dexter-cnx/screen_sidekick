use sidekick_core::{Annotation, EffectBrushStyle, Point};

pub const DEFAULT_EFFECT_BRUSH_RADIUS: f32 = 18.0;
pub const DEFAULT_EFFECT_BRUSH_STRENGTH: f32 = 0.65;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectBrushKind {
    Blur,
    Pixelate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectBrushDraft {
    kind: EffectBrushKind,
    points: Vec<Point>,
    style: EffectBrushStyle,
}

impl EffectBrushDraft {
    pub fn new(kind: EffectBrushKind, radius: f32, strength: f32) -> Self {
        Self {
            kind,
            points: Vec::new(),
            style: EffectBrushStyle::new(radius, strength),
        }
    }

    pub fn with_defaults(kind: EffectBrushKind) -> Self {
        Self::new(
            kind,
            DEFAULT_EFFECT_BRUSH_RADIUS,
            DEFAULT_EFFECT_BRUSH_STRENGTH,
        )
    }

    pub fn kind(&self) -> EffectBrushKind {
        self.kind
    }

    pub fn points(&self) -> &[Point] {
        &self.points
    }

    pub fn style(&self) -> &EffectBrushStyle {
        &self.style
    }

    pub fn push_point(&mut self, point: Point) -> bool {
        let minimum_spacing = (self.style.radius * 0.15).max(0.5);
        if self
            .points
            .last()
            .is_some_and(|last| distance(*last, point) < minimum_spacing)
        {
            return false;
        }

        self.points.push(point);
        true
    }

    pub fn commit(self) -> Option<Annotation> {
        if self.points.is_empty() {
            return None;
        }

        Some(match self.kind {
            EffectBrushKind::Blur => Annotation::BlurBrush {
                points: self.points,
                style: self.style,
            },
            EffectBrushKind::Pixelate => Annotation::PixelateBrush {
                points: self.points,
                style: self.style,
            },
        })
    }
}

fn distance(a: Point, b: Point) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blur_draft_commits_platform_neutral_annotation() {
        let mut draft = EffectBrushDraft::with_defaults(EffectBrushKind::Blur);
        assert!(draft.push_point(Point { x: 10.0, y: 20.0 }));
        assert!(draft.push_point(Point { x: 20.0, y: 30.0 }));

        assert!(matches!(
            draft.commit(),
            Some(Annotation::BlurBrush { points, style })
                if points.len() == 2
                    && style.radius == DEFAULT_EFFECT_BRUSH_RADIUS
                    && style.strength == DEFAULT_EFFECT_BRUSH_STRENGTH
        ));
    }

    #[test]
    fn pixelate_draft_commits_platform_neutral_annotation() {
        let mut draft = EffectBrushDraft::new(EffectBrushKind::Pixelate, 24.0, 0.8);
        assert!(draft.push_point(Point { x: 4.0, y: 8.0 }));

        assert_eq!(
            draft.commit(),
            Some(Annotation::PixelateBrush {
                points: vec![Point { x: 4.0, y: 8.0 }],
                style: EffectBrushStyle::new(24.0, 0.8),
            })
        );
    }

    #[test]
    fn sampling_rejects_points_that_are_too_close() {
        let mut draft = EffectBrushDraft::new(EffectBrushKind::Blur, 20.0, 0.5);
        assert!(draft.push_point(Point { x: 0.0, y: 0.0 }));
        assert!(!draft.push_point(Point { x: 1.0, y: 1.0 }));
        assert!(draft.push_point(Point { x: 4.0, y: 0.0 }));
        assert_eq!(draft.points().len(), 2);
    }

    #[test]
    fn empty_draft_does_not_commit() {
        let draft = EffectBrushDraft::with_defaults(EffectBrushKind::Pixelate);
        assert_eq!(draft.commit(), None);
    }
}
