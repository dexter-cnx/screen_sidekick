use crate::{Annotation, Point, SavedCapture, SidecarDocument};
use std::{collections::BTreeSet, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseImage {
    path: std::path::PathBuf,
    width: u32,
    height: u32,
    hash: String,
}

impl BaseImage {
    pub fn from_saved_capture(capture: SavedCapture, hash: impl Into<String>) -> Self {
        Self {
            path: capture.path,
            width: capture.width,
            height: capture.height,
            hash: hash.into(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }
}

#[derive(Debug, Clone)]
struct EditorSnapshot {
    annotations: Vec<Annotation>,
    selection: BTreeSet<usize>,
}

#[derive(Debug, Clone)]
pub struct EditorDocument {
    base: BaseImage,
    sidecar: SidecarDocument,
    selection: BTreeSet<usize>,
    undo_stack: Vec<EditorSnapshot>,
    redo_stack: Vec<EditorSnapshot>,
}

impl EditorDocument {
    pub fn new(base: BaseImage) -> Self {
        let sidecar = SidecarDocument::empty(base.hash().to_owned());
        Self {
            base,
            sidecar,
            selection: BTreeSet::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn from_sidecar(base: BaseImage, sidecar: SidecarDocument) -> Option<Self> {
        if !sidecar.is_supported_version() || sidecar.base_hash != base.hash() {
            return None;
        }

        Some(Self {
            base,
            sidecar,
            selection: BTreeSet::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        })
    }

    pub fn base(&self) -> &BaseImage {
        &self.base
    }

    pub fn sidecar(&self) -> &SidecarDocument {
        &self.sidecar
    }

    pub fn annotations(&self) -> &[Annotation] {
        &self.sidecar.annotations
    }

    pub fn add_annotation(&mut self, annotation: Annotation) -> usize {
        self.record_history();
        self.sidecar.push(annotation);
        self.sidecar.annotations.len() - 1
    }

    pub fn selected_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.selection.iter().copied()
    }

    pub fn select_only(&mut self, index: usize) -> bool {
        if index >= self.sidecar.annotations.len() {
            return false;
        }
        self.selection.clear();
        self.selection.insert(index);
        true
    }

    pub fn toggle_selection(&mut self, index: usize) -> bool {
        if index >= self.sidecar.annotations.len() {
            return false;
        }
        if !self.selection.remove(&index) {
            self.selection.insert(index);
        }
        true
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    pub fn translate_selected(&mut self, dx: f32, dy: f32) -> usize {
        if self.selection.is_empty() || (dx == 0.0 && dy == 0.0) {
            return 0;
        }
        self.record_history();
        let mut changed = 0;
        for index in self.selection.iter().copied() {
            if let Some(annotation) = self.sidecar.annotations.get_mut(index) {
                translate_annotation(annotation, dx, dy);
                changed += 1;
            }
        }
        changed
    }

    pub fn scale_selected(&mut self, origin: Point, scale_x: f32, scale_y: f32) -> usize {
        if self.selection.is_empty()
            || scale_x <= 0.0
            || scale_y <= 0.0
            || (scale_x == 1.0 && scale_y == 1.0)
        {
            return 0;
        }
        self.record_history();
        let mut changed = 0;
        for index in self.selection.iter().copied() {
            if let Some(annotation) = self.sidecar.annotations.get_mut(index) {
                scale_annotation(annotation, origin, scale_x, scale_y);
                changed += 1;
            }
        }
        changed
    }

    pub fn delete_selected(&mut self) -> usize {
        if self.selection.is_empty() {
            return 0;
        }

        self.record_history();
        let selected = std::mem::take(&mut self.selection);
        let original_len = self.sidecar.annotations.len();
        self.sidecar.annotations = self
            .sidecar
            .annotations
            .drain(..)
            .enumerate()
            .filter_map(|(index, annotation)| (!selected.contains(&index)).then_some(annotation))
            .collect();
        original_len - self.sidecar.annotations.len()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.undo_stack.pop() else {
            return false;
        };
        self.redo_stack.push(self.snapshot());
        self.restore_snapshot(previous);
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo_stack.pop() else {
            return false;
        };
        self.undo_stack.push(self.snapshot());
        self.restore_snapshot(next);
        true
    }

    fn snapshot(&self) -> EditorSnapshot {
        EditorSnapshot {
            annotations: self.sidecar.annotations.clone(),
            selection: self.selection.clone(),
        }
    }

    fn restore_snapshot(&mut self, snapshot: EditorSnapshot) {
        self.sidecar.annotations = snapshot.annotations;
        self.selection = snapshot.selection;
    }

    fn record_history(&mut self) {
        self.undo_stack.push(self.snapshot());
        self.redo_stack.clear();
    }
}

fn translate_point(point: &mut Point, dx: f32, dy: f32) {
    point.x += dx;
    point.y += dy;
}

fn scale_point(point: &mut Point, origin: Point, scale_x: f32, scale_y: f32) {
    point.x = origin.x + (point.x - origin.x) * scale_x;
    point.y = origin.y + (point.y - origin.y) * scale_y;
}

fn translate_annotation(annotation: &mut Annotation, dx: f32, dy: f32) {
    match annotation {
        Annotation::Rectangle { x, y, .. }
        | Annotation::Ellipse { x, y, .. }
        | Annotation::Text { x, y, .. }
        | Annotation::NumberMarker { x, y, .. } => {
            *x += dx;
            *y += dy;
        }
        Annotation::Line { start, end, .. } | Annotation::Arrow { start, end, .. } => {
            translate_point(start, dx, dy);
            translate_point(end, dx, dy);
        }
        Annotation::Freehand { points, .. }
        | Annotation::BlurBrush { points, .. }
        | Annotation::PixelateBrush { points, .. } => {
            for point in points {
                translate_point(point, dx, dy);
            }
        }
    }
}

fn scale_annotation(annotation: &mut Annotation, origin: Point, scale_x: f32, scale_y: f32) {
    match annotation {
        Annotation::Rectangle { x, y, w, h, .. } | Annotation::Ellipse { x, y, w, h, .. } => {
            let mut top_left = Point { x: *x, y: *y };
            scale_point(&mut top_left, origin, scale_x, scale_y);
            *x = top_left.x;
            *y = top_left.y;
            *w *= scale_x;
            *h *= scale_y;
        }
        Annotation::Line { start, end, .. } | Annotation::Arrow { start, end, .. } => {
            scale_point(start, origin, scale_x, scale_y);
            scale_point(end, origin, scale_x, scale_y);
        }
        Annotation::Freehand { points, .. }
        | Annotation::BlurBrush { points, .. }
        | Annotation::PixelateBrush { points, .. } => {
            for point in points {
                scale_point(point, origin, scale_x, scale_y);
            }
        }
        Annotation::Text { x, y, .. } | Annotation::NumberMarker { x, y, .. } => {
            let mut anchor = Point { x: *x, y: *y };
            scale_point(&mut anchor, origin, scale_x, scale_y);
            *x = anchor.x;
            *y = anchor.y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AnnotationStyle, EffectBrushStyle};
    use std::path::PathBuf;

    fn base() -> BaseImage {
        BaseImage::from_saved_capture(
            SavedCapture {
                path: PathBuf::from("capture.png"),
                width: 800,
                height: 600,
            },
            "sha256:base",
        )
    }

    fn rectangle(x: f32) -> Annotation {
        Annotation::Rectangle {
            x,
            y: 2.0,
            w: 30.0,
            h: 40.0,
            style: AnnotationStyle {
                stroke: "#ff0000".to_owned(),
                stroke_width: 2.0,
                fill: None,
            },
        }
    }

    #[test]
    fn base_image_is_exposed_read_only_and_sidecar_tracks_hash() {
        let document = EditorDocument::new(base());
        assert_eq!(document.base().path(), Path::new("capture.png"));
        assert_eq!(document.base().width(), 800);
        assert_eq!(document.base().height(), 600);
        assert_eq!(document.sidecar().base_hash, "sha256:base");
    }

    #[test]
    fn sidecar_must_match_base_hash_and_version() {
        let mut sidecar = SidecarDocument::empty("sha256:other");
        assert!(EditorDocument::from_sidecar(base(), sidecar.clone()).is_none());
        sidecar.base_hash = "sha256:base".to_owned();
        assert!(EditorDocument::from_sidecar(base(), sidecar.clone()).is_some());
        sidecar.v += 1;
        assert!(EditorDocument::from_sidecar(base(), sidecar).is_none());
    }

    #[test]
    fn selection_supports_multi_move_resize_and_delete() {
        let mut document = EditorDocument::new(base());
        document.add_annotation(rectangle(1.0));
        document.add_annotation(rectangle(2.0));
        document.add_annotation(rectangle(3.0));
        assert!(document.select_only(0));
        assert!(document.toggle_selection(2));
        assert_eq!(document.translate_selected(10.0, 5.0), 2);
        assert_eq!(
            document.scale_selected(Point { x: 0.0, y: 0.0 }, 2.0, 2.0),
            2
        );
        assert_eq!(document.delete_selected(), 2);
        assert_eq!(document.annotations().len(), 1);
    }

    #[test]
    fn effect_brush_points_participate_in_editor_transforms() {
        let style = EffectBrushStyle::new(12.0, 0.5);
        let mut blur = Annotation::BlurBrush {
            points: vec![Point { x: 2.0, y: 3.0 }, Point { x: 4.0, y: 5.0 }],
            style: style.clone(),
        };
        translate_annotation(&mut blur, 10.0, -1.0);
        scale_annotation(&mut blur, Point { x: 0.0, y: 0.0 }, 2.0, 3.0);
        assert!(matches!(
            blur,
            Annotation::BlurBrush { ref points, .. }
                if points[0] == Point { x: 24.0, y: 6.0 }
                    && points[1] == Point { x: 28.0, y: 12.0 }
        ));

        let mut pixelate = Annotation::PixelateBrush {
            points: vec![Point { x: 1.0, y: 2.0 }],
            style,
        };
        translate_annotation(&mut pixelate, 2.0, 3.0);
        assert!(matches!(
            pixelate,
            Annotation::PixelateBrush { ref points, .. }
                if points[0] == Point { x: 3.0, y: 5.0 }
        ));
    }

    #[test]
    fn undo_and_redo_restore_annotation_snapshots() {
        let mut document = EditorDocument::new(base());
        document.add_annotation(rectangle(1.0));
        document.select_only(0);
        document.translate_selected(10.0, 0.0);
        assert!(document.can_undo());
        assert!(document.undo());
        assert!(matches!(document.annotations()[0], Annotation::Rectangle { x, .. } if x == 1.0));
        assert!(document.can_redo());
        assert!(document.redo());
        assert!(matches!(document.annotations()[0], Annotation::Rectangle { x, .. } if x == 11.0));
    }

    #[test]
    fn undo_and_redo_restore_selection_identity_by_snapshot() {
        let mut document = EditorDocument::new(base());
        document.add_annotation(rectangle(1.0));
        document.add_annotation(rectangle(2.0));
        document.add_annotation(rectangle(3.0));
        document.select_only(1);
        document.delete_selected();
        document.select_only(1);

        assert!(document.undo());
        assert_eq!(document.selected_indices().collect::<Vec<_>>(), vec![1]);
        assert!(matches!(document.annotations()[1], Annotation::Rectangle { x, .. } if x == 2.0));

        assert!(document.redo());
        assert_eq!(document.selected_indices().collect::<Vec<_>>(), vec![1]);
        assert!(matches!(document.annotations()[1], Annotation::Rectangle { x, .. } if x == 3.0));
    }

    #[test]
    fn new_mutation_clears_redo_history() {
        let mut document = EditorDocument::new(base());
        document.add_annotation(rectangle(1.0));
        document.undo();
        assert!(document.can_redo());
        document.add_annotation(rectangle(2.0));
        assert!(!document.can_redo());
    }

    #[test]
    fn invalid_selection_indices_are_rejected() {
        let mut document = EditorDocument::new(base());
        document.add_annotation(rectangle(1.0));
        assert!(!document.select_only(1));
        assert!(!document.toggle_selection(1));
        assert!(document.selected_indices().next().is_none());
    }
}
