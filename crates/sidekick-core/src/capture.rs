use crate::{CaptureFrame, SavedCapture};
use chrono::Local;
use image::RgbaImage;
use std::{path::PathBuf, thread, time::Duration};
use thiserror::Error;
use xcap::{Monitor, Window};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureRegion {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl CaptureRegion {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Result<Self, CaptureError> {
        let x = u32::try_from(x).map_err(|_| CaptureError::InvalidRegion)?;
        let y = u32::try_from(y).map_err(|_| CaptureError::InvalidRegion)?;
        if width == 0 || height == 0 {
            return Err(CaptureError::InvalidRegion);
        }

        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }

    pub fn x(self) -> u32 {
        self.x
    }

    pub fn y(self) -> u32 {
        self.y
    }

    pub fn width(self) -> u32 {
        self.width
    }

    pub fn height(self) -> u32 {
        self.height
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum WindowShadowPolicy {
    #[default]
    Include,
    Exclude,
}

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
    #[error("capture region must have non-negative coordinates and non-zero dimensions")]
    InvalidRegion,
    #[error("invalid RGBA buffer")]
    InvalidImage,
    #[error("image I/O error: {0}")]
    Image(#[from] image::ImageError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub trait Capturer: Send + Sync {
    fn capture_fullscreen(&self, delay: Duration) -> Result<CaptureFrame, CaptureError>;
    fn capture_focused_window(
        &self,
        delay: Duration,
        shadow_policy: WindowShadowPolicy,
    ) -> Result<CaptureFrame, CaptureError>;
    fn available_windows(&self) -> Result<Vec<CaptureWindow>, CaptureError>;
    fn capture_window(
        &self,
        window_id: u32,
        delay: Duration,
        shadow_policy: WindowShadowPolicy,
    ) -> Result<CaptureFrame, CaptureError>;
    fn capture_region(
        &self,
        region: CaptureRegion,
        delay: Duration,
    ) -> Result<CaptureFrame, CaptureError>;
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

    fn capture_window_image(
        window: &Window,
        shadow_policy: WindowShadowPolicy,
    ) -> Result<RgbaImage, CaptureError> {
        let image = window.capture_image()?;
        let expected_width = window.width()?;
        let expected_height = window.height()?;
        Ok(apply_window_shadow_policy(
            image,
            expected_width,
            expected_height,
            shadow_policy,
        ))
    }
}

fn apply_window_shadow_policy(
    image: RgbaImage,
    expected_width: u32,
    expected_height: u32,
    shadow_policy: WindowShadowPolicy,
) -> RgbaImage {
    if shadow_policy == WindowShadowPolicy::Include
        || image.width() <= expected_width
        || image.height() <= expected_height
    {
        return image;
    }

    let crop_width = expected_width.min(image.width());
    let crop_height = expected_height.min(image.height());
    let x = (image.width() - crop_width) / 2;
    let y = (image.height() - crop_height) / 2;
    image::imageops::crop_imm(&image, x, y, crop_width, crop_height).to_image()
}

impl Capturer for XcapCapturer {
    fn capture_fullscreen(&self, delay: Duration) -> Result<CaptureFrame, CaptureError> {
        Self::wait(delay);
        let image = Self::primary_monitor()?.capture_image()?;
        Ok(Self::frame_from_image(image))
    }

    fn capture_focused_window(
        &self,
        delay: Duration,
        shadow_policy: WindowShadowPolicy,
    ) -> Result<CaptureFrame, CaptureError> {
        Self::wait(delay);
        let window = Self::focused_window()?;
        let image = Self::capture_window_image(&window, shadow_policy)?;
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

    fn capture_window(
        &self,
        window_id: u32,
        delay: Duration,
        shadow_policy: WindowShadowPolicy,
    ) -> Result<CaptureFrame, CaptureError> {
        Self::wait(delay);
        let window = Self::window_by_id(window_id)?;
        let image = Self::capture_window_image(&window, shadow_policy)?;
        Ok(Self::frame_from_image(image))
    }

    fn capture_region(
        &self,
        region: CaptureRegion,
        delay: Duration,
    ) -> Result<CaptureFrame, CaptureError> {
        Self::wait(delay);
        let image = Self::primary_monitor()?.capture_region(
            region.x(),
            region.y(),
            region.width(),
            region.height(),
        )?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn capture_region_accepts_positive_area() {
        let region = CaptureRegion::new(10, 20, 300, 200).expect("region should be valid");
        assert_eq!(region.x(), 10);
        assert_eq!(region.y(), 20);
        assert_eq!(region.width(), 300);
        assert_eq!(region.height(), 200);
    }

    #[test]
    fn capture_region_rejects_invalid_geometry() {
        assert!(CaptureRegion::new(-1, 0, 10, 10).is_err());
        assert!(CaptureRegion::new(0, -1, 10, 10).is_err());
        assert!(CaptureRegion::new(0, 0, 0, 10).is_err());
        assert!(CaptureRegion::new(0, 0, 10, 0).is_err());
    }

    #[test]
    fn include_shadow_preserves_captured_dimensions() {
        let image = RgbaImage::from_pixel(120, 90, Rgba([1, 2, 3, 255]));
        let result = apply_window_shadow_policy(image, 100, 70, WindowShadowPolicy::Include);
        assert_eq!(result.dimensions(), (120, 90));
    }

    #[test]
    fn exclude_shadow_crops_to_window_pixel_bounds() {
        let image = RgbaImage::from_pixel(120, 90, Rgba([1, 2, 3, 255]));
        let result = apply_window_shadow_policy(image, 100, 70, WindowShadowPolicy::Exclude);
        assert_eq!(result.dimensions(), (100, 70));
    }
}
