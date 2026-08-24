use crate::{CaptureFrame, SavedCapture};
use chrono::Local;
use image::RgbaImage;
use std::{path::PathBuf, thread, time::Duration};
use thiserror::Error;
use xcap::{Monitor, Window};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureWindow {
    pub id: u32,
    pub app_name: String,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub is_focused: bool,
}

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("xcap error: {0}")]
    Xcap(#[from] xcap::XCapError),
    #[error("no display was found")]
    NoDisplay,
    #[error("no focused window was found")]
    NoFocusedWindow,
    #[error("window {0} was not found")]
    WindowNotFound(u32),
    #[error("invalid RGBA buffer")]
    InvalidImage,
    #[error("image I/O error: {0}")]
    Image(#[from] image::ImageError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub trait Capturer: Send + Sync {
    fn capture_fullscreen(&self, delay: Duration) -> Result<CaptureFrame, CaptureError>;
    fn capture_focused_window(&self, delay: Duration) -> Result<CaptureFrame, CaptureError>;
    fn available_windows(&self) -> Result<Vec<CaptureWindow>, CaptureError>;
    fn capture_window(&self, window_id: u32, delay: Duration) -> Result<CaptureFrame, CaptureError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct XcapCapturer;

impl XcapCapturer {
    fn primary_monitor() -> Result<Monitor, CaptureError> {
        let monitors = Monitor::all()?;
        monitors
            .iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .cloned()
            .or_else(|| monitors.into_iter().next())
            .ok_or(CaptureError::NoDisplay)
    }

    fn focused_window() -> Result<Window, CaptureError> {
        Self::capturable_windows()?
            .into_iter()
            .find(|window| window.is_focused().unwrap_or(false))
            .ok_or(CaptureError::NoFocusedWindow)
    }

    fn capturable_windows() -> Result<Vec<Window>, CaptureError> {
        Ok(Window::all()?
            .into_iter()
            .filter(|window| {
                !window.is_minimized().unwrap_or(true)
                    && window.width().unwrap_or_default() > 0
                    && window.height().unwrap_or_default() > 0
            })
            .collect())
    }

    fn window_by_id(window_id: u32) -> Result<Window, CaptureError> {
        Self::capturable_windows()?
            .into_iter()
            .find(|window| window.id().ok() == Some(window_id))
            .ok_or(CaptureError::WindowNotFound(window_id))
    }

    fn wait(delay: Duration) {
        if !delay.is_zero() {
            thread::sleep(delay);
        }
    }

    fn frame_from_image(image: RgbaImage) -> CaptureFrame {
        CaptureFrame {
            width: image.width(),
            height: image.height(),
            rgba: image.into_raw(),
        }
    }
}

impl Capturer for XcapCapturer {
    fn capture_fullscreen(&self, delay: Duration) -> Result<CaptureFrame, CaptureError> {
        Self::wait(delay);
        let image = Self::primary_monitor()?.capture_image()?;
        Ok(Self::frame_from_image(image))
    }

    fn capture_focused_window(&self, delay: Duration) -> Result<CaptureFrame, CaptureError> {
        Self::wait(delay);
        let image = Self::focused_window()?.capture_image()?;
        Ok(Self::frame_from_image(image))
    }

    fn available_windows(&self) -> Result<Vec<CaptureWindow>, CaptureError> {
        Self::capturable_windows()?
            .into_iter()
            .map(|window| {
                Ok(CaptureWindow {
                    id: window.id()?,
                    app_name: window.app_name()?,
                    title: window.title()?,
                    width: window.width()?,
                    height: window.height()?,
                    is_focused: window.is_focused()?,
                })
            })
            .collect()
    }

    fn capture_window(&self, window_id: u32, delay: Duration) -> Result<CaptureFrame, CaptureError> {
        Self::wait(delay);
        let image = Self::window_by_id(window_id)?.capture_image()?;
        Ok(Self::frame_from_image(image))
    }
}

impl CaptureFrame {
    pub fn save_quick_png(&self) -> Result<SavedCapture, CaptureError> {
        let root = dirs::picture_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("Screen Sidekick");
        std::fs::create_dir_all(&root)?;

        let filename = format!("Sidekick-{}.png", Local::now().format("%Y%m%d-%H%M%S"));
        let path: PathBuf = root.join(filename);
        let image = RgbaImage::from_raw(self.width, self.height, self.rgba.clone())
            .ok_or(CaptureError::InvalidImage)?;
        image.save(&path)?;

        Ok(SavedCapture {
            path,
            width: self.width,
            height: self.height,
        })
    }
}
