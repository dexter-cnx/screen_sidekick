mod runtime;

use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use gpui::{App, AsyncApp, WindowHandle, prelude::*};
use gpui_platform::application;
use runtime::AppRuntime;
use sidekick_core::{Capturer, PreviewStack, XcapCapturer};
use sidekick_ui::{OverlayCard, overlay_window_options};
use std::{path::PathBuf, sync::mpsc, time::Duration};
use tray_icon::menu::MenuEvent;

const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(50);

struct PreviewWindow {
    handle: WindowHandle<OverlayCard>,
}

fn rebuild_preview_windows(
    cx: &mut AsyncApp,
    preview_stack: &PreviewStack,
    preview_windows: &mut Vec<PreviewWindow>,
    delete_sender: &mpsc::Sender<PathBuf>,
) {
    for preview in preview_windows.drain(..) {
        cx.update(|cx| {
            let _ = preview
                .handle
                .update(cx, |_, window, _| window.remove_window());
        });
    }

    let stack_size = preview_stack.len();
    for (stack_slot, capture) in preview_stack.items().cloned().enumerate() {
        let delete_sender = delete_sender.clone();
        let handle = cx.update(|cx| {
            cx.open_window(
                overlay_window_options(cx, stack_slot),
                move |_, cx| {
                    cx.new(|_| OverlayCard::new(capture, stack_size, delete_sender))
                },
            )
            .expect("failed to open Screen Sidekick overlay")
        });

        preview_windows.push(PreviewWindow { handle });
    }
}

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
            let mut preview_windows: Vec<PreviewWindow> = Vec::new();
            let (delete_sender, delete_receiver) = mpsc::channel();

            loop {
                let mut capture_requested = false;

                let mut stack_changed = false;
                while let Ok(path) = delete_receiver.try_recv() {
                    let index = preview_stack
                        .items()
                        .position(|capture| capture.path == path);
                    if let Some(index) = index {
                        preview_stack.remove(index);
                        stack_changed = true;
                    }
                }

                if stack_changed {
                    rebuild_preview_windows(
                        cx,
                        &preview_stack,
                        &mut preview_windows,
                        &delete_sender,
                    );
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
                            rebuild_preview_windows(
                                cx,
                                &preview_stack,
                                &mut preview_windows,
                                &delete_sender,
                            );
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
