# Screen Sidekick Roadmap

## Platform policy

Screen Sidekick is **macOS-first**. All MVP implementation, product iteration, integration testing, packaging, and stabilization target macOS 13+ first.

Windows 10/11 is a planned follow-up platform. During the macOS-first phase, Windows is not a release target and is not required in CI. Platform-neutral traits and module boundaries must still be preserved so Windows can be implemented later without restructuring the core product.

## M0 — Capture-to-overlay vertical slice (macOS)
- [x] Cargo workspace
- [x] `Capturer` abstraction
- [x] xcap fullscreen capture
- [x] PNG quick save
- [x] transparent GPUI floating overlay
- [x] bottom-right placement
- [x] sidecar v1 model seed
- [x] macOS compile + Clippy verification
- [x] rustfmt clean
- [ ] native macOS always-on-top/exclude-from-capture hooks

## M1 — Tray + hotkeys + preview stack (macOS)
- [x] tray icon and menu foundation
- [x] Option+1 fullscreen hotkey registration
- [x] tray/hotkey event dispatch into GPUI controller
- [ ] global hotkeys: window/area
- [x] configurable key recorder model
- [x] wire configurable hotkey model into runtime registration
- [x] settings/key-recorder UI
- [x] bounded preview stack model, max visible count, newest-on-top
- [x] connect preview stack to GPUI overlay rendering
- [x] preview stack reflow after capture/delete mutations
- [x] auto-dismiss -> peek visibility state machine model
- [x] runtime auto-dismiss timer + peek tab rendering/activation
- [x] clipboard copy
- [x] delete action
- [x] quick-save status

## M2 — Capture modes + native window behavior (macOS)
- [x] focused-window capture via xcap
- [x] window chooser capture
- [x] platform-neutral area capture contract + xcap region backend
- [x] interactive area selector
- [x] 0/3/5 second timer
- [ ] include/exclude shadow policy
- [ ] all-spaces/floating/excluded-from-capture behavior
- [ ] click-through only while collapsed

## M3 — Annotation foundation
- immutable base image
- `.sidekick.json` versioned sidecar
- selection/move/resize/multi-select
- rectangle/filled rect/ellipse/line/arrow/freehand
- text/number markers
- undo/redo/delete

## M4 — Advanced annotation + export
- blur/pixelate brush
- highlight dimmer
- PNG/JPEG export
- JPEG quality slider
- clipboard render
- Quick Save

## M5 — History + settings
- thumbnail grid
- date search/filter
- reveal in Finder
- retention/delete
- General/Screenshots/Hotkeys/Overlay settings
- launch at startup

## M6 — macOS productization
- app icon and macOS metadata
- signed/notarized macOS app
- Homebrew cask
- release CI
- real-device/workflow stabilization

## Platform Phase — Windows 10/11
Begin only after the macOS product path is stable.

- enable Windows CI
- validate/build GPUI Windows backend against the pinned revision
- xcap DXGI/WGC capture implementation and behavior validation
- native topmost/layered/tool-window/excluded-from-capture hooks via `windows-rs`
- Explorer integration/history reveal behavior
- launch-at-startup implementation
- Windows packaging/signing
- Scoop manifest
- platform parity test matrix

Windows work must reuse the existing core/domain contracts rather than introducing Windows behavior into `sidekick-core`.

## Phase 2 — Window Sidekick (macOS first)
- snap active window: halves, quarters, thirds, maximize, center
- customizable global hotkeys
- multi-monitor movement and layout awareness
- macOS implementation via Accessibility APIs
- custom window layouts
- drag-to-snap layout overlay

After the macOS Window Sidekick implementation is stable, add a Windows implementation via Win32 / `windows-rs` behind the same `WindowManager` boundary.

## Phase 2.5 — Workspace Presets
- save and restore multi-window layouts
- per-app placement rules
- named presets such as Coding / Meeting / Presentation
- optional capture-after-arrange workflow

Window management stays outside the MVP screenshot path and behind a platform-neutral `WindowManager` boundary so it cannot couple capture or annotation logic to OS APIs.
