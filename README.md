# Screen Sidekick

**Capture. Annotate. Ship.**

Screen Sidekick is a Rust + GPUI screenshot utility targeting macOS 13+ and Windows 10/11.

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

### Windows 10/11

Requirements:

- latest stable Rust (MSVC toolchain)
- Visual Studio Build Tools with Desktop development with C++
- Windows 10/11 SDK

```powershell
cargo run -p screen-sidekick
```

`xcap` is built with its WGC feature enabled. The GPUI dependency is pinned to the same Zed revision for macOS and Windows to avoid API drift.

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

`sidekick-core` must not depend on GPUI. Platform-specific behavior stays behind traits/modules so a future alternate frontend does not require rewriting capture/history/editing logic.

## M0 verification

Launching the binary performs one fullscreen capture, writes it to the quick-save directory, then opens a transparent floating preview with the thumbnail and image dimensions.

## Next milestone

M1 should move capture triggering to tray/global hotkeys and add a bounded preview stack. See `docs/ROADMAP.md`.

## Packaging target

Homebrew/Scoop commands will be added once signed/notarized macOS and Windows release artifacts exist. Publishing install commands before those artifacts exist would be misleading.
