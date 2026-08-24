# Area Capture Foundation

This slice adds the platform-neutral capture contract needed by the interactive area selector without coupling `sidekick-core` to GPUI.

## Core contract

`CaptureRegion` represents monitor-local pixel geometry:

- `x`, `y`: non-negative monitor-local origin
- `width`, `height`: non-zero pixel dimensions

Construction through `CaptureRegion::new` rejects negative origins and zero-sized regions before a capture backend is invoked.

`Capturer::capture_region` accepts a `CaptureRegion` plus the existing delay argument. The contract is reusable by GPUI today and a future Flutter frontend because it contains no UI types.

## xcap backend

`XcapCapturer` resolves the primary monitor and delegates to `Monitor::capture_region`. xcap performs final monitor-bound validation and returns an error when the requested rectangle extends outside the monitor.

The resulting image is normalized into the existing `CaptureFrame`, so quick-save and preview-stack handling can reuse the same pipeline as fullscreen and window capture.

## Next slice

The interactive selector will live in `sidekick-ui` as a transparent GPUI selection surface. It will translate pointer drag geometry into `CaptureRegion` and send that value back to `sidekick-app`, which will dispatch it through the existing background capture pipeline.
