# M3 Text and Number Marker Notes

## Number markers

- Click-to-place on the immutable base-image coordinate system.
- Marker numbers continue from the highest existing marker number in the editor document.
- Marker foreground/background/diameter remain sidecar-owned via `MarkerStyle`.
- Rendering converts base-image coordinates through the same contained-image geometry as the other annotation tools.

## Text

Text annotations already exist in sidecar v1. The GPUI editor UI must use a real focused text-input path rather than committing placeholder strings. The implementation should preserve IME/composition semantics and commit the final text to `Annotation::Text` at the selected base-image position.
