use anyhow::Context as _;
use gpui::{App, prelude::*};
use gpui_platform::application;
use sidekick_core::{Capturer, XcapCapturer};
use sidekick_ui::{OverlayCard, overlay_window_options};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // M0 smoke flow: launch -> fullscreen capture -> quick-save -> floating preview.
    // Hotkeys/tray will move capture triggering out of startup in M1.
    let saved = XcapCapturer
        .capture_fullscreen(Duration::ZERO)
        .context("fullscreen capture failed")?
        .save_quick_png()
        .context("saving capture failed")?;

    application().run(move |cx: &mut App| {
        let capture = saved.clone();
        cx.open_window(overlay_window_options(cx), move |_, cx| {
            cx.new(|_| OverlayCard::new(capture))
        })
        .expect("failed to open Screen Sidekick overlay");
    });

    Ok(())
}
