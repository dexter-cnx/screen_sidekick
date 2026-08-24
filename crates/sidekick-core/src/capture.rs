use crate::{CaptureFrame, SavedCapture};
use chrono::Local;
use image::RgbaImage;
use std::{path::PathBuf, thread, time::Duration};
use thiserror::Error;
use xcap::Monitor;

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("xcap error: {0}")]
    Xcap(#[from] xcap::XCapError),
    #[error("no display was found")]
    NoDisplay,
    #[error("invalid RGBA buffer")]
    InvalidImage,
    #[error("image I/O error: {0}")]
    Image(#[from] image::ImageError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub trait Capturer: Send + Sync {
    fn capture_fullscreen(&self, delay: Duration) -> Result<CaptureFrame, CaptureError>;
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
}

impl Capturer for XcapCapturer {
    fn capture_fullscreen(&self, delay: Duration) -> Result<CaptureFrame, CaptureError> {
        if !delay.is_zero() {
            thread::sleep(delay);
        }

        let image = Self::primary_monitor()?.capture_image()?;
        Ok(CaptureFrame {
            width: image.width(),
            height: image.height(),
            rgba: image.into_raw(),
        })
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
