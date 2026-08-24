mod runtime;

use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use gpui::{App, prelude::*};
use gpui_platform::application;
use runtime::AppRuntime;
use sidekick_core::{Capturer, PreviewStack, XcapCapturer};
use sidekick_ui::{OverlayCard, overlay_window_options};
use std::{sync::mpsc, time::Duration};
use tray_icon::menu::MenuEvent;

const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(50);

fn main() -> anyhow::Result<()> {
    application().run(|cx: &mut App| {
        // Both tray-icon and global-hotkey require the macOS event loop to be running
        // on the main thread, so they are intentionally initialized inside GPUI's run closure.
        let runtime = AppRuntime::new().expect("failed to initialize tray/hotkey runtime");
        let capture_menu_id = runtime.capture_menu_id().clone();
        let quit_menu_id = runtime.quit_menu_id().clone();
        let fullscreen_hotkey_id = runtime.fullscreen_hotkey_id();

        cx.spawn(async move |cx| {
            // Keep runtime resources alive for the whole dispatch task lifetime.
            let _runtime = runtime;
            let mut preview_stack = PreviewStack::default();
            let (delete_sender, delete_receiver) = mpsc::channel();

            loop {
                let mut capture_requested = false;

                while let Ok(path) = delete_receiver.try_recv() {
                    if let Some(index) = preview_stack
                        .items()
                        .position(|capture| capture.path == path)
                    {
                        preview_stack.remove(index);
                    }
                }

                while let Ok(event) = MenuEvent::receiver().try_recv() {
                    if event.id == quit_menu_id {
                        cx.update(|cx| cx.quit());
                        return;
                    }
                    if event.id == capture_menu_id {
                        capture_requested = true;
                    }
                }

                while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                    if event.id == fullscreen_hotkey_id && event.state == HotKeyState::Pressed {
                        capture_requested = true;
                    }
                }

                if capture_requested {
                    let capture_result = cx
                        .background_spawn(async move {
                            XcapCapturer
                                .capture_fullscreen(Duration::ZERO)?
                                .save_quick_png()
                        })
                        .await;

                    match capture_result {
                        Ok(saved) => {
                            preview_stack.push(saved);
                            let stack_size = preview_stack.len();
                            let capture = preview_stack
                                .newest()
                                .expect("preview stack must contain the capture just pushed")
                                .clone();
                            let delete_sender = delete_sender.clone();

                            cx.update(|cx| {
                                cx.open_window(overlay_window_options(cx), move |_, cx| {
                                    cx.new(|_| OverlayCard::new(capture, stack_size, delete_sender))
                                })
                                .expect("failed to open Screen Sidekick overlay");
                            });
                        }
                        Err(error) => eprintln!("Screen Sidekick capture failed: {error:#}"),
                    }
                }

                cx.background_executor().timer(EVENT_POLL_INTERVAL).await;
            }
        })
        .detach();
    });

    Ok(())
}
