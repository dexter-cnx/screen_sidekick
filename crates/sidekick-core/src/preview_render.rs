use std::path::Path;

use image::ImageFormat;

use crate::{Annotation, render_annotations_with_text};

pub fn save_annotation_preview(
    base_path: impl AsRef<Path>,
    annotations: &[Annotation],
    output_path: impl AsRef<Path>,
) -> Result<(), image::ImageError> {
    let base = image::open(base_path)?.to_rgba8();
    let rendered = render_annotations_with_text(&base, annotations);
    rendered.save_with_format(output_path, ImageFormat::Png)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use image::{Rgba, RgbaImage};

    use super::*;
    use crate::{EffectBrushStyle, Point};

    #[test]
    fn writes_a_derived_preview_without_touching_the_base() {
        let root = std::env::temp_dir().join(format!(
            "screen-sidekick-preview-render-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let base_path = root.join("base.png");
        let preview_path = root.join("preview.png");
        let base = RgbaImage::from_fn(24, 24, |x, y| {
            Rgba([(x * 9) as u8, (y * 9) as u8, ((x + y) * 4) as u8, 255])
        });
        base.save(&base_path).unwrap();
        let base_bytes = fs::read(&base_path).unwrap();

        save_annotation_preview(
            &base_path,
            &[Annotation::PixelateBrush {
                points: vec![Point { x: 12.0, y: 12.0 }],
                style: EffectBrushStyle::new(6.0, 1.0),
            }],
            &preview_path,
        )
        .unwrap();

        assert!(preview_path.exists());
        assert_eq!(fs::read(&base_path).unwrap(), base_bytes);
        assert_ne!(fs::read(&preview_path).unwrap(), base_bytes);
        let _ = fs::remove_dir_all(root);
    }
}
