# Screen Sidekick Roadmap

## M0 — Capture-to-overlay vertical slice
- [x] Cargo workspace
- [x] `Capturer` abstraction
- [x] xcap fullscreen capture
- [x] PNG quick save
- [x] transparent GPUI floating overlay
- [x] bottom-right placement
- [x] sidecar v1 model seed
- [ ] macOS/Windows CI verification
- [ ] native always-on-top/exclude-from-capture hooks

## M1 — Tray + hotkeys + preview stack
- tray icon and menu
- global hotkeys: fullscreen/window/area
- configurable key recorder model
- preview card stack, max visible count, newest-on-top
- auto-dismiss -> peek tab state machine
- clipboard copy
- delete/save actions

## M2 — Capture modes + native window behavior
- focused/window chooser capture
- interactive area selector
- 0/3/5 second timer
- include/exclude shadow platform policy
- macOS: all-spaces/floating/excluded-from-capture behavior
- Windows: topmost/layered/tool-window/excluded-from-capture behavior
- click-through only while collapsed

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
- reveal in Finder/Explorer
- retention/delete
- General/Screenshots/Hotkeys/Overlay settings
- launch at startup

## M6 — Productization
- app icon and platform metadata
- signed/notarized macOS app
- signed Windows package
- Homebrew cask
- Scoop manifest
- release CI matrix

## Phase 2 — Window Sidekick
- snap active window: halves, quarters, thirds, maximize, center
- customizable global hotkeys
- multi-monitor movement and layout awareness
- macOS implementation via Accessibility APIs
- Windows implementation via Win32 / windows-rs
- custom window layouts
- drag-to-snap layout overlay

## Phase 2.5 — Workspace Presets
- save and restore multi-window layouts
- per-app placement rules
- named presets such as Coding / Meeting / Presentation
- optional capture-after-arrange workflow

Window management stays outside the MVP screenshot path and behind a platform-neutral `WindowManager` boundary so it cannot couple capture or annotation logic to OS APIs.
