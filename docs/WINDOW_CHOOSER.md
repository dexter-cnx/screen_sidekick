# Window Chooser Capture

## Flow

1. The tray exposes **Choose Window…**.
2. `sidekick-app` asks `XcapCapturer::available_windows()` for capturable windows.
3. `sidekick-core` returns platform-neutral `CaptureWindow` descriptors containing only window metadata and IDs.
4. `sidekick-ui::WindowChooserView` renders those descriptors and sends the selected window ID back to the app runtime.
5. The runtime dispatches `CaptureRequest::Window(id)`.
6. `XcapCapturer::capture_window(id, delay)` resolves the current xcap window and captures it.
7. The resulting frame uses the same quick-save and preview-stack pipeline as fullscreen and focused-window capture.

## Boundary

`sidekick-core` owns window enumeration and capture contracts but has no GPUI dependency. The chooser is a frontend concern in `sidekick-ui`. A future Flutter frontend can request the same `CaptureWindow` list and call the same capture-by-ID API without reworking the capture layer.

## Selection policy

The core list excludes minimized windows and zero-sized windows. xcap returns windows in z-order, so the chooser preserves that order. Focus state is exposed as metadata to help the UI identify the currently focused candidate.

## Follow-up

- add richer chooser presentation/preview if needed
- interactive area selector
- configurable window/area hotkeys once those capture modes are stable
