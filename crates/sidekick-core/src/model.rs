use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    Fullscreen,
    Window,
    Area,
}

#[derive(Debug, Clone)]
pub struct CaptureOptions {
    pub mode: CaptureMode,
    pub include_shadows: bool,
    pub timer_seconds: u8,
}

impl Default for CaptureOptions {
    fn default() -> Self {
        Self {
            mode: CaptureMode::Fullscreen,
            include_shadows: true,
            timer_seconds: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CaptureFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SavedCapture {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
}
