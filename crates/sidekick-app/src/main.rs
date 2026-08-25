mod capture_hotkeys;
mod preferences;
mod runtime;

use capture_hotkeys::CaptureHotkeys;
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use gpui::{App, AsyncApp, WindowHandle, prelude::*};
use gpui_platform::application;
use runtime::AppRuntime;
use sidekick_core::{
    CaptureRegion, Capturer, PreviewStack, PreviewVisibility, PreviewVisibilityState,
    WindowShadowPolicy, XcapCapturer,
};
use sidekick_ui::{
    AreaSelectorView, HotkeySettingsView, OverlayCard, PeekTab, WindowChooserView,
    area_selector_window_options, overlay_window_options, peek_window_options,
    settings_window_options, window_chooser_options,
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
    Window(u32),
    Area(CaptureRegion),
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

fn open_area_selector(
    cx: &mut AsyncApp,
    area_sender: &mpsc::Sender<CaptureRegion>,
    area_selector_window: &mut Option<WindowHandle<AreaSelectorView>>,
) {
    if let Some(handle) = area_selector_window.take() {
        cx.update(|cx| {
            let _ = handle.update(cx, |_, window, _| window.remove_window());
        });
    }
    let area_sender = area_sender.clone();
    let handle = cx.update(|cx| {
        cx.open_window(area_selector_window_options(cx), move |_, cx| {
            cx.new(|_| AreaSelectorView::new(area_sender))
        })
        .expect("failed to open Screen Sidekick area selector")
    });
    *area_selector_window = Some(handle);
}

fn main() -> anyhow::Result<()> {
    application().run(|cx: &mut App| {
        let mut runtime = AppRuntime::new().expect("failed to initialize tray/hotkey runtime");
        let capture_hotkeys = CaptureHotkeys::new();
        let window_hotkey_id = capture_hotkeys.window_hotkey_id();
        let area_hotkey_id = capture_hotkeys.area_hotkey_id();
        runtime.set_window_shadow_policy(preferences::load_window_shadow_policy());
        let capture_menu_id = runtime.capture_menu_id().clone();
        let capture_window_menu_id = runtime.capture_window_menu_id().clone();
        let choose_window_menu_id = runtime.choose_window_menu_id().clone();
        let capture_area_menu_id = runtime.capture_area_menu_id().clone();
        let timer_zero_menu_id = runtime.timer_zero_menu_id().clone();
        let timer_three_menu_id = runtime.timer_three_menu_id().clone();
        let timer_five_menu_id = runtime.timer_five_menu_id().clone();
        let shadow_include_menu_id = runtime.shadow_include_menu_id().clone();
        let shadow_exclude_menu_id = runtime.shadow_exclude_menu_id().clone();
        let settings_menu_id = runtime.settings_menu_id().clone();
        let quit_menu_id = runtime.quit_menu_id().clone();

        cx.spawn(async move |cx| {
            let _capture_hotkeys = capture_hotkeys;
            let mut runtime = runtime;
            let mut preview_stack = PreviewStack::default();
            let mut preview_visibility = PreviewVisibilityState::default();
            let mut preview_windows: Vec<PreviewWindow> = Vec::new();
            let mut peek_window: Option<WindowHandle<PeekTab>> = None;
            let mut settings_window: Option<WindowHandle<HotkeySettingsView>> = None;
            let mut chooser_window: Option<WindowHandle<WindowChooserView>> = None;
            let mut area_selector_window: Option<WindowHandle<AreaSelectorView>> = None;
            let mut auto_dismiss_at: Option<Instant> = None;
            let (delete_sender, delete_receiver) = mpsc::channel();
            let (peek_sender, peek_receiver) = mpsc::channel();
            let (binding_sender, binding_receiver) = mpsc::channel();
            let (window_sender, window_receiver) = mpsc::channel();
            let (area_sender, area_receiver) = mpsc::channel();

            loop {
                let mut capture_request = None;

                while let Ok(window_id) = window_receiver.try_recv() {
                    chooser_window = None;
                    capture_request = Some(CaptureRequest::Window(window_id));
                }

                while let Ok(region) = area_receiver.try_recv() {
                    area_selector_window = None;
                    capture_request = Some(CaptureRequest::Area(region));
                }

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
                    if event.id == timer_zero_menu_id {
                        runtime.set_capture_delay(Duration::ZERO);
                    }
                    if event.id == timer_three_menu_id {
                        runtime.set_capture_delay(Duration::from_secs(3));
                    }
                    if event.id == timer_five_menu_id {
                        runtime.set_capture_delay(Duration::from_secs(5));
                    }
                    if event.id == shadow_include_menu_id {
                        runtime.set_window_shadow_policy(WindowShadowPolicy::Include);
                        preferences::save_window_shadow_policy(WindowShadowPolicy::Include);
                    }
                    if event.id == shadow_exclude_menu_id {
                        runtime.set_window_shadow_policy(WindowShadowPolicy::Exclude);
                        preferences::save_window_shadow_policy(WindowShadowPolicy::Exclude);
                    }
                    if event.id == capture_menu_id {
                        capture_request = Some(CaptureRequest::Fullscreen);
                    }
                    if event.id == capture_window_menu_id {
                        capture_request = Some(CaptureRequest::FocusedWindow);
                    }
                    if event.id == choose_window_menu_id {
                        if let Some(handle) = chooser_window.take() {
                            cx.update(|cx| {
                                let _ = handle.update(cx, |_, window, _| window.remove_window());
                            });
                        }

                        match XcapCapturer.available_windows() {
                            Ok(windows) => {
                                let window_sender = window_sender.clone();
                                let handle = cx.update(|cx| {
                                    cx.open_window(window_chooser_options(cx), move |_, cx| {
                                        cx.new(|_| WindowChooserView::new(windows, window_sender))
                                    })
                                    .expect("failed to open Screen Sidekick window chooser")
                                });
                                chooser_window = Some(handle);
                            }
                            Err(error) => {
                                eprintln!("Screen Sidekick window listing failed: {error:#}");
                            }
                        }
                    }
                    if event.id == capture_area_menu_id {
                        open_area_selector(cx, &area_sender, &mut area_selector_window);
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
                    if event.state != HotKeyState::Pressed {
                        continue;
                    }
                    if event.id == runtime.fullscreen_hotkey_id() {
                        capture_request = Some(CaptureRequest::Fullscreen);
                    } else if Some(event.id) == window_hotkey_id {
                        capture_request = Some(CaptureRequest::FocusedWindow);
                    } else if Some(event.id) == area_hotkey_id {
                        open_area_selector(cx, &area_sender, &mut area_selector_window);
                    }
                }

                if let Some(request) = capture_request {
                    let delay = runtime.capture_delay();
                    let shadow_policy = runtime.window_shadow_policy();
                    remove_preview_windows(cx, &mut preview_windows);
                    remove_peek_window(cx, &mut peek_window);
                    auto_dismiss_at = None;

                    let capture_result = cx
                        .background_spawn(async move {
                            let frame = match request {
                                CaptureRequest::Fullscreen => {
                                    XcapCapturer.capture_fullscreen(delay)?
                                }
                                CaptureRequest::FocusedWindow => XcapCapturer
                                    .capture_focused_window_with_shadow(delay, shadow_policy)?,
                                CaptureRequest::Window(window_id) => XcapCapturer
                                    .capture_window_with_shadow(window_id, delay, shadow_policy)?,
                                CaptureRequest::Area(region) => {
                                    XcapCapturer.capture_region(region, delay)?
                                }
                            };
                            frame.save_quick_png()
                        })
                        .await;

                    match capture_result {
                        Ok(saved) => {
                            preview_stack.push(saved);
                            preview_visibility.on_capture();
                            rebuild_preview_windows(
                                cx,
                                &preview_stack,
                                &mut preview_windows,
                                &delete_sender,
                            );
                            auto_dismiss_at = Some(Instant::now() + PREVIEW_AUTO_DISMISS);
                        }
                        Err(error) => {
                            eprintln!("Screen Sidekick capture failed: {error:#}");
                            if !preview_stack.is_empty()
                                && preview_visibility.visibility() == PreviewVisibility::Expanded
                            {
                                rebuild_preview_windows(
                                    cx,
                                    &preview_stack,
                                    &mut preview_windows,
                                    &delete_sender,
                                );
                                auto_dismiss_at = Some(Instant::now() + PREVIEW_AUTO_DISMISS);
                            }
                        }
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
