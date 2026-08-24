mod runtime;

use anyhow::Context as _;
use gpui::{App, prelude::*};
use gpui_platform::application;
use runtime::AppRuntime;
use sidekick_core::{Capturer, XcapCapturer};
use sidekick_ui::{OverlayCard, overlay_window_options};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // Keep the M0 capture-on-launch smoke flow until M1 event dispatch is wired.
    // This ensures every intermediate M1 commit still exercises capture + overlay.
    let saved = XcapCapturer
        .capture_fullscreen(Duration::ZERO)
        .context("fullscreen capture failed")?
        .save_quick_png()
        .context("saving capture failed")?;

    application().run(move |cx: &mut App| {
        // Both tray-icon and global-hotkey require the macOS event loop to be running
        // on the main thread, so they are intentionally initialized inside GPUI's run closure.
        let runtime = AppRuntime::new().expect("failed to initialize tray/hotkey runtime");

        let capture = saved.clone();
        cx.open_window(overlay_window_options(cx), move |_, cx| {
            cx.new(|_| OverlayCard::new(capture))
        })
        .expect("failed to open Screen Sidekick overlay");

        // Runtime resources must remain alive for the full application lifetime.
        // M1 event dispatch will move this ownership into the application controller.
        std::mem::forget(runtime);
    });

    Ok(())
}
