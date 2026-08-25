use gpui::Window;

pub fn apply_overlay_window_behavior(window: &Window) {
    #[cfg(target_os = "macos")]
    apply_macos_overlay_window_behavior(window);
}

#[cfg(target_os = "macos")]
fn apply_macos_overlay_window_behavior(window: &Window) {
    use objc2_app_kit::{
        NSFloatingWindowLevel, NSWindow, NSWindowCollectionBehavior, NSWindowSharingType,
    };
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let Ok(handle) = window.window_handle() else {
        return;
    };
    let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
        return;
    };

    let ns_window = unsafe { &*handle.ns_window.as_ptr().cast::<NSWindow>() };
    ns_window.setLevel(unsafe { NSFloatingWindowLevel });

    let mut behavior = ns_window.collectionBehavior();
    behavior.insert(NSWindowCollectionBehavior::CanJoinAllSpaces);
    behavior.insert(NSWindowCollectionBehavior::FullScreenAuxiliary);
    ns_window.setCollectionBehavior(behavior);
    ns_window.setSharingType(NSWindowSharingType::None);
}
