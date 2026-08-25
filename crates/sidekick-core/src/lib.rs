pub mod capture;
pub mod editor;
pub mod hotkey;
pub mod model;
pub mod preview;
pub mod sidecar;

pub use capture::{CaptureRegion, CaptureWindow, Capturer, WindowShadowPolicy, XcapCapturer};
pub use editor::{BaseImage, EditorDocument};
pub use hotkey::{HotkeyAction, HotkeyBinding, HotkeyKey, HotkeyModifiers, HotkeyValidationError};
pub use model::{CaptureFrame, CaptureMode, CaptureOptions, SavedCapture};
pub use preview::{DEFAULT_PREVIEW_LIMIT, PreviewStack, PreviewVisibility, PreviewVisibilityState};
pub use sidecar::{
    Annotation, AnnotationStyle, MarkerStyle, Point, SIDECAR_VERSION, SidecarDocument, TextStyle,
};
