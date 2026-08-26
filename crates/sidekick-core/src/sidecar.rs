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
    BlurBrush {
        points: Vec<Point>,
        style: EffectBrushStyle,
    },
    PixelateBrush {
        points: Vec<Point>,
        style: EffectBrushStyle,
    },
    HighlightDimmer {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        style: DimmerStyle,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectBrushStyle {
    pub radius: f32,
    pub strength: f32,
}

impl EffectBrushStyle {
    pub fn new(radius: f32, strength: f32) -> Self {
        Self {
            radius: radius.max(1.0),
            strength: strength.clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimmerStyle {
    pub color: String,
    pub opacity: f32,
}

impl DimmerStyle {
    pub fn new(color: impl Into<String>, opacity: f32) -> Self {
        Self {
            color: color.into(),
            opacity: opacity.clamp(0.0, 1.0),
        }
    }
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
        assert!(matches!(
            document.annotations[0],
            Annotation::Rectangle { .. }
        ));
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

    #[test]
    fn m4_effect_brush_annotations_round_trip() {
        let mut document = SidecarDocument::empty("sha256:base");
        let points = vec![Point { x: 4.0, y: 8.0 }, Point { x: 12.0, y: 16.0 }];
        document.push(Annotation::BlurBrush {
            points: points.clone(),
            style: EffectBrushStyle::new(18.0, 0.65),
        });
        document.push(Annotation::PixelateBrush {
            points,
            style: EffectBrushStyle::new(24.0, 0.8),
        });

        let json = serde_json::to_string(&document).unwrap();
        let decoded: SidecarDocument = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, document);
    }

    #[test]
    fn highlight_dimmer_annotation_round_trips() {
        let mut document = SidecarDocument::empty("sha256:base");
        document.push(Annotation::HighlightDimmer {
            x: 10.0,
            y: 20.0,
            w: 300.0,
            h: 120.0,
            style: DimmerStyle::new("#000000", 0.55),
        });

        let json = serde_json::to_string(&document).unwrap();
        let decoded: SidecarDocument = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, document);
        assert!(json.contains("\"type\":\"highlight_dimmer\""));
    }

    #[test]
    fn effect_brush_style_normalizes_radius_and_strength() {
        assert_eq!(
            EffectBrushStyle::new(0.0, 2.0),
            EffectBrushStyle {
                radius: 1.0,
                strength: 1.0,
            }
        );
        assert_eq!(
            EffectBrushStyle::new(12.0, -1.0),
            EffectBrushStyle {
                radius: 12.0,
                strength: 0.0,
            }
        );
    }

    #[test]
    fn dimmer_style_normalizes_opacity() {
        assert_eq!(
            DimmerStyle::new("#000000", 2.0),
            DimmerStyle {
                color: "#000000".to_owned(),
                opacity: 1.0,
            }
        );
        assert_eq!(
            DimmerStyle::new("#101010", -0.5),
            DimmerStyle {
                color: "#101010".to_owned(),
                opacity: 0.0,
            }
        );
    }
}
