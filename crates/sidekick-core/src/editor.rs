use crate::{Annotation, SavedCapture, SidecarDocument};
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
pub struct EditorDocument {
    base: BaseImage,
    sidecar: SidecarDocument,
    selection: BTreeSet<usize>,
}

impl EditorDocument {
    pub fn new(base: BaseImage) -> Self {
        let sidecar = SidecarDocument::empty(base.hash().to_owned());
        Self {
            base,
            sidecar,
            selection: BTreeSet::new(),
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

    pub fn delete_selected(&mut self) -> usize {
        if self.selection.is_empty() {
            return 0;
        }

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AnnotationStyle;
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
    fn selection_supports_single_multi_toggle_and_delete() {
        let mut document = EditorDocument::new(base());
        document.add_annotation(rectangle(1.0));
        document.add_annotation(rectangle(2.0));
        document.add_annotation(rectangle(3.0));

        assert!(document.select_only(0));
        assert!(document.toggle_selection(2));
        assert_eq!(document.selected_indices().collect::<Vec<_>>(), vec![0, 2]);

        assert_eq!(document.delete_selected(), 2);
        assert_eq!(document.annotations().len(), 1);
        assert!(document.selected_indices().next().is_none());
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
