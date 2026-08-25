use serde::{Deserialize, Serialize};

pub const SIDECAR_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SidecarDocument {
    pub v: u32,
    pub base_hash: String,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Annotation {
    Rectangle {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        style: AnnotationStyle,
    },
    Ellipse {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        style: AnnotationStyle,
    },
    Line {
        start: Point,
        end: Point,
        style: AnnotationStyle,
    },
    Arrow {
        start: Point,
        end: Point,
        style: AnnotationStyle,
    },
    Freehand {
        points: Vec<Point>,
        style: AnnotationStyle,
    },
    Text {
        x: f32,
        y: f32,
        text: String,
        style: TextStyle,
    },
    NumberMarker {
        x: f32,
        y: f32,
        number: u32,
        style: MarkerStyle,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationStyle {
    pub stroke: String,
    pub stroke_width: f32,
    pub fill: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    pub color: String,
    pub font_size: f32,
    pub background: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkerStyle {
    pub foreground: String,
    pub background: String,
    pub diameter: f32,
}

impl SidecarDocument {
    pub fn empty(base_hash: impl Into<String>) -> Self {
        Self {
            v: SIDECAR_VERSION,
            base_hash: base_hash.into(),
            annotations: Vec::new(),
        }
    }

    pub fn push(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }

    pub fn is_supported_version(&self) -> bool {
        self.v == SIDECAR_VERSION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape_style() -> AnnotationStyle {
        AnnotationStyle {
            stroke: "#ff0000".to_owned(),
            stroke_width: 3.0,
            fill: Some("#22000000".to_owned()),
        }
    }

    #[test]
    fn empty_document_uses_current_version_and_preserves_base_hash() {
        let document = SidecarDocument::empty("sha256:base-image");

        assert_eq!(document.v, SIDECAR_VERSION);
        assert_eq!(document.base_hash, "sha256:base-image");
        assert!(document.annotations.is_empty());
        assert!(document.is_supported_version());
    }

    #[test]
    fn legacy_rectangle_json_remains_compatible() {
        let json = r##"{
            "v": 1,
            "base_hash": "sha256:legacy",
            "annotations": [{
                "type": "rectangle",
                "x": 10.0,
                "y": 20.0,
                "w": 100.0,
                "h": 50.0,
                "style": {
                    "stroke": "#ff0000",
                    "stroke_width": 2.0,
                    "fill": null
                }
            }]
        }"##;

        let document: SidecarDocument = serde_json::from_str(json).unwrap();

        assert_eq!(document.v, 1);
        assert_eq!(document.annotations.len(), 1);
        assert!(matches!(document.annotations[0], Annotation::Rectangle { .. }));
    }

    #[test]
    fn all_m3_annotation_shapes_round_trip() {
        let mut document = SidecarDocument::empty("sha256:base");
        let style = shape_style();
        document.push(Annotation::Rectangle {
            x: 1.0,
            y: 2.0,
            w: 30.0,
            h: 40.0,
            style: style.clone(),
        });
        document.push(Annotation::Ellipse {
            x: 5.0,
            y: 6.0,
            w: 70.0,
            h: 80.0,
            style: style.clone(),
        });
        document.push(Annotation::Line {
            start: Point { x: 1.0, y: 1.0 },
            end: Point { x: 10.0, y: 10.0 },
            style: style.clone(),
        });
        document.push(Annotation::Arrow {
            start: Point { x: 2.0, y: 3.0 },
            end: Point { x: 20.0, y: 30.0 },
            style: style.clone(),
        });
        document.push(Annotation::Freehand {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 4.0, y: 8.0 }],
            style,
        });
        document.push(Annotation::Text {
            x: 12.0,
            y: 18.0,
            text: "note".to_owned(),
            style: TextStyle {
                color: "#ffffff".to_owned(),
                font_size: 16.0,
                background: Some("#88000000".to_owned()),
            },
        });
        document.push(Annotation::NumberMarker {
            x: 50.0,
            y: 60.0,
            number: 1,
            style: MarkerStyle {
                foreground: "#ffffff".to_owned(),
                background: "#ff0000".to_owned(),
                diameter: 24.0,
            },
        });

        let json = serde_json::to_string(&document).unwrap();
        let decoded: SidecarDocument = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, document);
    }
}
