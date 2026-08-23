# Architecture Notes

## Platform strategy

Screen Sidekick is implemented and stabilized on macOS first. Windows is deliberately deferred until the macOS product path is stable.

This is a delivery decision, not an excuse to couple the codebase to macOS. Core/domain APIs must remain platform-neutral and platform-specific behavior must stay behind cfg-gated modules or traits.

## Capture boundary

The `Capturer` trait is intentionally UI-free. M0 exposes fullscreen capture; later modes can be added without GPUI entering `sidekick-core`.

The active implementation currently targets macOS. A future Windows capturer should satisfy the same domain contract rather than introduce Windows conditionals into callers.

## GPUI window model

Current GPUI uses `focus`, `WindowKind`, and `WindowBackgroundAppearance`; it does not expose the prompt's proposed `focusable` and `always_on_top` fields on `WindowOptions`.

M0 therefore uses:

- `focus: false`
- `WindowKind::Floating`
- `WindowBackgroundAppearance::Transparent`
- no titlebar
- fixed bounds at the primary display's lower-right corner

Native window policy should be layered on after the window exists. During the active macOS phase, native hooks use macOS APIs only. Those hooks are platform implementation details and must not leak into the capture/core crate API.

## Windows readiness

Windows is a planned platform, not a currently supported one.

The repository may retain dependencies, feature flags, traits, and cfg boundaries that make Windows implementation easier later, but Windows-specific runtime behavior, CI, packaging, and parity work are deferred until macOS is stable.

When Windows work begins, expected platform integrations include xcap DXGI/WGC capture behavior and Win32 native window hooks via `windows-rs`.

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

Platform implementations belong behind cfg-gated modules. macOS will be implemented first using Accessibility APIs. A future Windows implementation will use Win32 via `windows-rs`. GPUI renders selectors/settings only and must not own OS window manipulation.
