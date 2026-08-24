use crate::{CaptureFrame, SavedCapture};
use chrono::Local;
#[cfg(target_os = "macos")]
use core_graphics::{
    base::kCGImageAlphaPremultipliedLast,
    color_space::CGColorSpace,
    context::CGContext,
    display::{CGDisplay, CGRectNull},
    geometry::{CGPoint, CGRect, CGSize},
    image::CGImage,
    window::{
        kCGWindowImageBestResolution, kCGWindowImageBoundsIgnoreFraming,
        kCGWindowListOptionIncludingWindow,
    },
};
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
    #[error("window shadow exclusion is not supported by the current capture backend")]
    ShadowExclusionUnsupported,
    #[error("native window capture failed for window {0}")]
    NativeWindowCaptureFailed(u32),
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
    fn capture_focused_window(&self, delay: Duration) -> Result<CaptureFrame, CaptureError>;
    fn capture_focused_window_with_shadow(
        &self,
        delay: Duration,
        shadow_policy: WindowShadowPolicy,
    ) -> Result<CaptureFrame, CaptureError>;
    fn available_windows(&self) -> Result<Vec<CaptureWindow>, CaptureError>;
    fn capture_window(&self, window_id: u32, delay: Duration)
    -> Result<CaptureFrame, CaptureError>;
    fn capture_window_with_shadow(
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
        match shadow_policy {
            WindowShadowPolicy::Include => Ok(window.capture_image()?),
            WindowShadowPolicy::Exclude => {
                #[cfg(target_os = "macos")]
                {
                    capture_macos_window_without_shadow(window.id()?)
                }
                #[cfg(not(target_os = "macos"))]
                {
                    Err(CaptureError::ShadowExclusionUnsupported)
                }
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn capture_macos_window_without_shadow(window_id: u32) -> Result<RgbaImage, CaptureError> {
    let image = CGDisplay::screenshot(
        unsafe { CGRectNull },
        kCGWindowListOptionIncludingWindow,
        window_id,
        kCGWindowImageBoundsIgnoreFraming | kCGWindowImageBestResolution,
    )
    .ok_or(CaptureError::NativeWindowCaptureFailed(window_id))?;

    cg_image_to_rgba(&image)
}

#[cfg(target_os = "macos")]
fn cg_image_to_rgba(image: &CGImage) -> Result<RgbaImage, CaptureError> {
    let width = image.width();
    let height = image.height();
    let bytes_per_row = width.checked_mul(4).ok_or(CaptureError::InvalidImage)?;
    let color_space = CGColorSpace::create_device_rgb();
    let mut context = CGContext::create_bitmap_context(
        None,
        width,
        height,
        8,
        bytes_per_row,
        &color_space,
        kCGImageAlphaPremultipliedLast,
    );

    context.translate(0.0, height as f64);
    context.scale(1.0, -1.0);
    context.draw_image(
        CGRect::new(
            &CGPoint::new(0.0, 0.0),
            &CGSize::new(width as f64, height as f64),
        ),
        image,
    );

    let mut rgba = context.data().to_vec();
    unpremultiply_rgba(&mut rgba);
    RgbaImage::from_raw(width as u32, height as u32, rgba).ok_or(CaptureError::InvalidImage)
}

#[cfg(target_os = "macos")]
fn unpremultiply_rgba(rgba: &mut [u8]) {
    for pixel in rgba.chunks_exact_mut(4) {
        let alpha = u32::from(pixel[3]);
        if alpha == 0 || alpha == 255 {
            continue;
        }

        for channel in &mut pixel[..3] {
            let value = (u32::from(*channel) * 255 + alpha / 2) / alpha;
            *channel = value.min(255) as u8;
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
        self.capture_focused_window_with_shadow(delay, WindowShadowPolicy::Include)
    }

    fn capture_focused_window_with_shadow(
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
    ) -> Result<CaptureFrame, CaptureError> {
        self.capture_window_with_shadow(window_id, delay, WindowShadowPolicy::Include)
    }

    fn capture_window_with_shadow(
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
    fn window_shadow_policy_defaults_to_include() {
        assert_eq!(WindowShadowPolicy::default(), WindowShadowPolicy::Include);
    }

    #[test]
    fn shadow_exclusion_error_is_explicit() {
        assert_eq!(
            CaptureError::ShadowExclusionUnsupported.to_string(),
            "window shadow exclusion is not supported by the current capture backend"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn unpremultiply_rgba_restores_straight_alpha_channels() {
        let mut rgba = [64, 32, 16, 128, 1, 2, 3, 255, 0, 0, 0, 0];
        unpremultiply_rgba(&mut rgba);
        assert_eq!(rgba, [128, 64, 32, 128, 1, 2, 3, 255, 0, 0, 0, 0]);
    }
}
