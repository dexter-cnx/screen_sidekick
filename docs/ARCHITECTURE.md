# Architecture Notes

## Capture boundary

The `Capturer` trait is intentionally UI-free. M0 exposes fullscreen capture; later modes can be added without GPUI entering `sidekick-core`.

## GPUI window model

Current GPUI uses `focus`, `WindowKind`, and `WindowBackgroundAppearance`; it does not expose the prompt's proposed `focusable` and `always_on_top` fields on `WindowOptions`.

M0 therefore uses:

- `focus: false`
- `WindowKind::Floating`
- `WindowBackgroundAppearance::Transparent`
- no titlebar
- fixed bounds at the primary display's lower-right corner

Native window policy should be layered on in M2 after the window exists. Those hooks are platform implementation details and must not leak into the capture/core crate API.

## Windows

xcap is enabled with WGC. Zed's current tree includes `gpui_windows`; Screen Sidekick pins GPUI/GPUI Platform to one Zed revision to keep the Windows and macOS backends aligned.

## Non-destructive model

The image remains the base asset. Sidecar documents are versioned and contain only annotation state. Rendering/export composes base + annotation commands into a new output image.

## Future window-management boundary

Window snapping is intentionally a separate capability from capture. The planned boundary is:

```rust
pub trait WindowManager {
    fn active_window(&self) -> anyhow::Result<WindowInfo>;
    fn move_resize(&self, window: &WindowInfo, rect: Rect) -> anyhow::Result<()>;
    fn move_to_display(&self, window: &WindowInfo, display: DisplayId) -> anyhow::Result<()>;
}
```

Platform implementations belong behind cfg-gated modules. macOS will use Accessibility APIs; Windows will use Win32 via `windows-rs`. GPUI renders selectors/settings only and must not own OS window manipulation.
