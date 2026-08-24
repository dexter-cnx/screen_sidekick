mod runtime;

use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use gpui::{App, AsyncApp, WindowHandle, prelude::*};
use gpui_platform::application;
use runtime::AppRuntime;
use sidekick_core::{
    Capturer, PreviewStack, PreviewVisibility, PreviewVisibilityState, XcapCapturer,
};
use sidekick_ui::{
    HotkeySettingsView, OverlayCard, PeekTab, overlay_window_options, peek_window_options,
    settings_window_options,
};
use std::{
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};
use tray_icon::menu::MenuEvent;

const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(50);
const PREVIEW_AUTO_DISMISS: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaptureRequest {
    Fullscreen,
    FocusedWindow,
}

struct PreviewWindow {
    handle: WindowHandle<OverlayCard>,
}

fn remove_preview_windows(cx: &mut AsyncApp, preview_windows: &mut Vec<PreviewWindow>) {
    for preview in preview_windows.drain(..) {
        cx.update(|cx| {
            let _ = preview
                .handle
                .update(cx, |_, window, _| window.remove_window());
        });
    }
}

fn rebuild_preview_windows(
    cx: &mut AsyncApp,
    preview_stack: &PreviewStack,
    preview_windows: &mut Vec<PreviewWindow>,
    delete_sender: &mpsc::Sender<PathBuf>,
) {
    remove_preview_windows(cx, preview_windows);

    let stack_size = preview_stack.len();
    let captures: Vec<_> = preview_stack.items().cloned().enumerate().collect();
    for (stack_slot, capture) in captures.into_iter().rev() {
        let delete_sender = delete_sender.clone();
        let handle = cx.update(|cx| {
            cx.open_window(overlay_window_options(cx, stack_slot), move |_, cx| {
                cx.new(|_| OverlayCard::new(capture, stack_size, delete_sender))
            })
            .expect("failed to open Screen Sidekick overlay")
        });

        preview_windows.push(PreviewWindow { handle });
    }
}

fn remove_peek_window(cx: &mut AsyncApp, peek_window: &mut Option<WindowHandle<PeekTab>>) {
    if let Some(handle) = peek_window.take() {
        cx.update(|cx| {
            let _ = handle.update(cx, |_, window, _| window.remove_window());
        });
    }
}

fn show_peek_window(
    cx: &mut AsyncApp,
    stack_size: usize,
    activate_sender: &mpsc::Sender<()>,
    peek_window: &mut Option<WindowHandle<PeekTab>>,
) {
    remove_peek_window(cx, peek_window);
    let activate_sender = activate_sender.clone();
    let handle = cx.update(|cx| {
        cx.open_window(peek_window_options(cx), move |_, cx| {
            cx.new(|_| PeekTab::new(stack_size, activate_sender))
        })
        .expect("failed to open Screen Sidekick peek tab")
    });
    *peek_window = Some(handle);
}

fn main() -> anyhow::Result<()> {
    application().run(|cx: &mut App| {
        let runtime = AppRuntime::new().expect("failed to initialize tray/hotkey runtime");
        let capture_menu_id = runtime.capture_menu_id().clone();
        let capture_window_menu_id = runtime.capture_window_menu_id().clone();
        let settings_menu_id = runtime.settings_menu_id().clone();
        let quit_menu_id = runtime.quit_menu_id().clone();

        cx.spawn(async move |cx| {
            let mut runtime = runtime;
            let mut preview_stack = PreviewStack::default();
            let mut preview_visibility = PreviewVisibilityState::default();
            let mut preview_windows: Vec<PreviewWindow> = Vec::new();
            let mut peek_window: Option<WindowHandle<PeekTab>> = None;
            let mut settings_window: Option<WindowHandle<HotkeySettingsView>> = None;
            let mut auto_dismiss_at: Option<Instant> = None;
            let (delete_sender, delete_receiver) = mpsc::channel();
            let (peek_sender, peek_receiver) = mpsc::channel();
            let (binding_sender, binding_receiver) = mpsc::channel();

            loop {
                let mut capture_request = None;

                while let Ok(binding) = binding_receiver.try_recv() {
                    let result = match runtime.set_fullscreen_binding(binding) {
                        Ok(()) => Ok(binding),
                        Err(error) => {
                            let message = format!("{error:#}");
                            eprintln!("Screen Sidekick hotkey update failed: {message}");
                            Err(message)
                        }
                    };

                    if let Some(handle) = settings_window.as_ref() {
                        cx.update(|cx| {
                            let _ = handle.update(cx, |view, _, cx| {
                                view.apply_binding_result(result, cx);
                            });
                        });
                    }
                }

                while peek_receiver.try_recv().is_ok() {
                    preview_visibility.on_peek_activated();
                    if preview_visibility.visibility() == PreviewVisibility::Expanded {
                        remove_peek_window(cx, &mut peek_window);
                        rebuild_preview_windows(
                            cx,
                            &preview_stack,
                            &mut preview_windows,
                            &delete_sender,
                        );
                        auto_dismiss_at = Some(Instant::now() + PREVIEW_AUTO_DISMISS);
                    }
                }

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
                    if preview_stack.is_empty() {
                        preview_visibility.on_stack_empty();
                        auto_dismiss_at = None;
                        remove_preview_windows(cx, &mut preview_windows);
                        remove_peek_window(cx, &mut peek_window);
                    } else if preview_visibility.visibility() == PreviewVisibility::Expanded {
                        rebuild_preview_windows(
                            cx,
                            &preview_stack,
                            &mut preview_windows,
                            &delete_sender,
                        );
                    }
                }

                while let Ok(event) = MenuEvent::receiver().try_recv() {
                    if event.id == quit_menu_id {
                        cx.update(|cx| cx.quit());
                        return;
                    }
                    if event.id == capture_menu_id {
                        capture_request = Some(CaptureRequest::Fullscreen);
                    }
                    if event.id == capture_window_menu_id {
                        capture_request = Some(CaptureRequest::FocusedWindow);
                    }
                    if event.id == settings_menu_id {
                        if let Some(handle) = settings_window.take() {
                            cx.update(|cx| {
                                let _ = handle.update(cx, |_, window, _| window.remove_window());
                            });
                        }
                        let binding = runtime.fullscreen_binding();
                        let binding_sender = binding_sender.clone();
                        let handle = cx.update(|cx| {
                            cx.open_window(settings_window_options(cx), move |window, cx| {
                                cx.new(|cx| {
                                    HotkeySettingsView::new(binding, binding_sender, window, cx)
                                })
                            })
                            .expect("failed to open Screen Sidekick settings")
                        });
                        settings_window = Some(handle);
                    }
                }

                while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                    if event.id == runtime.fullscreen_hotkey_id()
                        && event.state == HotKeyState::Pressed
                    {
                        capture_request = Some(CaptureRequest::Fullscreen);
                    }
                }

                if let Some(request) = capture_request {
                    let capture_result = cx
                        .background_spawn(async move {
                            let frame = match request {
                                CaptureRequest::Fullscreen => {
                                    XcapCapturer.capture_fullscreen(Duration::ZERO)?
                                }
                                CaptureRequest::FocusedWindow => {
                                    XcapCapturer.capture_focused_window(Duration::ZERO)?
                                }
                            };
                            frame.save_quick_png()
                        })
                        .await;

                    match capture_result {
                        Ok(saved) => {
                            preview_stack.push(saved);
                            preview_visibility.on_capture();
                            remove_peek_window(cx, &mut peek_window);
                            rebuild_preview_windows(
                                cx,
                                &preview_stack,
                                &mut preview_windows,
                                &delete_sender,
                            );
                            auto_dismiss_at = Some(Instant::now() + PREVIEW_AUTO_DISMISS);
                        }
                        Err(error) => eprintln!("Screen Sidekick capture failed: {error:#}"),
                    }
                }

                if preview_visibility.visibility() == PreviewVisibility::Expanded
                    && auto_dismiss_at.is_some_and(|deadline| Instant::now() >= deadline)
                {
                    preview_visibility.on_auto_dismiss();
                    auto_dismiss_at = None;
                    remove_preview_windows(cx, &mut preview_windows);
                    show_peek_window(cx, preview_stack.len(), &peek_sender, &mut peek_window);
                }

                cx.background_executor().timer(EVENT_POLL_INTERVAL).await;
            }
        })
        .detach();
    });

    Ok(())
}
