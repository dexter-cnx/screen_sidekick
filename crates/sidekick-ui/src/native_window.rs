use gpui::Window;

pub fn apply_overlay_window_behavior(window: &Window) {
    #[cfg(target_os = "macos")]
    apply_macos_overlay_window_behavior(window);
}

#[cfg(target_os = "macos")]
fn apply_macos_overlay_window_behavior(window: &Window) {
    use objc2_app_kit::{
        NSFloatingWindowLevel, NSView, NSWindowCollectionBehavior, NSWindowSharingType,
    };
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        return;
    };
    let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
        return;
    };

    let ns_view = unsafe { &*handle.ns_view.as_ptr().cast::<NSView>() };
    let Some(ns_window) = ns_view.window() else {
        return;
    };

    ns_window.setLevel(NSFloatingWindowLevel);

    let mut behavior = ns_window.collectionBehavior();
    behavior.insert(NSWindowCollectionBehavior::CanJoinAllSpaces);
    behavior.insert(NSWindowCollectionBehavior::FullScreenAuxiliary);
    ns_window.setCollectionBehavior(behavior);
    ns_window.setSharingType(NSWindowSharingType::None);
}
