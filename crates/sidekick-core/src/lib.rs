pub mod capture;
pub mod model;
pub mod preview;
pub mod sidecar;

pub use capture::{Capturer, XcapCapturer};
pub use model::{CaptureFrame, CaptureMode, CaptureOptions, SavedCapture};
pub use preview::{DEFAULT_PREVIEW_LIMIT, PreviewStack, PreviewVisibility, PreviewVisibilityState};
