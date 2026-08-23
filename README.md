# Screen Sidekick

**Capture. Annotate. Ship.**

Screen Sidekick is a Rust + GPUI screenshot utility.

## Platform strategy

**Active implementation target: macOS 13+.**

The product will be implemented, tested, and stabilized on macOS first. Windows 10/11 remains a planned platform, but Windows-specific implementation and CI are deferred until the macOS product path is stable.

The architecture still preserves platform boundaries so future Windows support does not require rewriting capture, history, annotation, or application-domain logic.

## Current milestone: M0

This scaffold intentionally implements only the first vertical slice:

- Cargo workspace: `sidekick-core`, `sidekick-ui`, `sidekick-app`
- capture abstraction through `Capturer`
- fullscreen capture through xcap
- native-resolution RGBA preservation
- quick-save PNG to `~/Pictures/Screen Sidekick/`
- transparent GPUI floating preview in the bottom-right
- sidecar v1 data model foundation
- platform dependencies isolated for future native window hooks

The overlay action buttons are visual placeholders in M0. Tray, global hotkeys, native click-through/exclude-from-capture hooks, card stack behavior, annotation, history, settings, and startup integration are deliberately staged next.

## Build

### macOS 13+

Requirements:

- latest stable Rust
- Xcode + Command Line Tools
- Screen Recording permission for the built app/terminal

```bash
xcode-select --install
cargo run -p screen-sidekick
```

### Windows 10/11 — planned

Windows support is intentionally deferred until the macOS implementation is stable. The core/domain boundaries should remain portable, while Windows capture, native window behavior, packaging, and CI will be implemented in a later platform phase.

Do not treat the current repository as Windows-supported yet.

## Architecture

```text
sidekick-core
  capture + sidecar + domain models
       ^
       |
sidekick-ui
  GPUI overlay/editor/history/settings
       ^
       |
sidekick-app
  lifecycle + tray + hotkeys + platform composition
```

`sidekick-core` must not depend on GPUI. Platform-specific behavior stays behind traits/modules so future Windows support or an alternate frontend does not require rewriting capture/history/editing logic.

## M0 verification

Launching the binary performs one fullscreen capture, writes it to the quick-save directory, then opens a transparent floating preview with the thumbnail and image dimensions.

During the macOS-first phase, required CI consists of Rust formatting, `cargo check`, and Clippy on macOS.

## Next milestone

M1 should move capture triggering to tray/global hotkeys and add a bounded preview stack. See `docs/ROADMAP.md`.

## Packaging target

Homebrew installation will be added after the macOS app is signed/notarized and release artifacts are stable. Scoop and Windows packaging are deferred with the Windows implementation phase.
