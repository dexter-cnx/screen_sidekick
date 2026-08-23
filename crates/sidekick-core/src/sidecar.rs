use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarDocument {
    pub v: u32,
    pub base_hash: String,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Annotation {
    Rectangle {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        style: AnnotationStyle,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationStyle {
    pub stroke: String,
    pub stroke_width: f32,
    pub fill: Option<String>,
}

impl SidecarDocument {
    pub fn empty(base_hash: impl Into<String>) -> Self {
        Self {
            v: 1,
            base_hash: base_hash.into(),
            annotations: Vec::new(),
        }
    }
}
