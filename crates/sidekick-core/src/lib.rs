pub mod annotation_renderer;
pub mod capture;
pub mod editor;
pub mod effect_compositor;
pub mod export;
pub mod hotkey;
pub mod model;
pub mod preview;
pub mod preview_render;
pub mod sidecar;
pub mod text_renderer;

pub use annotation_renderer::render_annotations;
pub use capture::{CaptureRegion, CaptureWindow, Capturer, WindowShadowPolicy, XcapCapturer};
pub use editor::{BaseImage, EditorDocument};
pub use effect_compositor::composite_effect_brushes;
pub use export::{
    DEFAULT_JPEG_QUALITY, ExportFormat, encode_annotation_export, encode_effect_composite,
    save_annotation_export, save_effect_composite,
};
pub use hotkey::{HotkeyAction, HotkeyBinding, HotkeyKey, HotkeyModifiers, HotkeyValidationError};
pub use model::{CaptureFrame, CaptureMode, CaptureOptions, SavedCapture};
pub use preview::{DEFAULT_PREVIEW_LIMIT, PreviewStack, PreviewVisibility, PreviewVisibilityState};
pub use preview_render::save_annotation_preview;
pub use sidecar::{
    Annotation, AnnotationStyle, DimmerStyle, EffectBrushStyle, MarkerStyle, Point,
    SIDECAR_VERSION, SidecarDocument, TextStyle,
};
pub use text_renderer::render_annotations_with_text;
