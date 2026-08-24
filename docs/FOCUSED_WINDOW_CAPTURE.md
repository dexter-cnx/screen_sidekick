# Focused Window Capture Walkthrough

This increment adds the first M2 capture mode without coupling `sidekick-core` to GPUI.

## Flow

1. The tray exposes **Capture Focused Window**.
2. `sidekick-app` converts the tray event into `CaptureRequest::FocusedWindow`.
3. The shared capture pipeline calls `XcapCapturer::capture_focused_window(Duration::ZERO)`.
4. `sidekick-core` enumerates xcap windows, selects the focused non-minimized window, and captures its image.
5. The resulting `CaptureFrame` uses the existing quick-save and preview-stack path.

## Boundaries

- `sidekick-core` owns capture behavior and xcap integration.
- `sidekick-app` owns runtime/tray dispatch.
- GPUI remains outside the core capture contract.
- Window chooser UI and area selection remain separate follow-up increments.

## Failure behavior

If xcap cannot resolve a focused non-minimized window, the core returns `CaptureError::NoFocusedWindow`. The runtime logs the capture failure and leaves the current preview stack unchanged.
