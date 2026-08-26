use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use image::{ImageEncoder, RgbaImage, codecs::jpeg::JpegEncoder, codecs::png::PngEncoder};

use crate::{Annotation, render_annotations_with_text};

pub const DEFAULT_JPEG_QUALITY: u8 = 90;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Png,
    Jpeg { quality: u8 },
}

impl ExportFormat {
    pub fn jpeg(quality: u8) -> Self {
        Self::Jpeg {
            quality: quality.clamp(1, 100),
        }
    }
}

impl Default for ExportFormat {
    fn default() -> Self {
        Self::Jpeg {
            quality: DEFAULT_JPEG_QUALITY,
        }
    }
}

pub fn encode_annotation_export(
    base_path: impl AsRef<Path>,
    annotations: &[Annotation],
    format: ExportFormat,
) -> Result<Vec<u8>, image::ImageError> {
    let base = image::open(base_path)?.to_rgba8();
    encode_effect_composite(&base, annotations, format)
}

pub fn save_annotation_export(
    base_path: impl AsRef<Path>,
    annotations: &[Annotation],
    format: ExportFormat,
    output_path: impl AsRef<Path>,
) -> Result<(), image::ImageError> {
    let base = image::open(base_path)?.to_rgba8();
    save_effect_composite(&base, annotations, format, output_path)
}

pub fn encode_effect_composite(
    base: &RgbaImage,
    annotations: &[Annotation],
    format: ExportFormat,
) -> Result<Vec<u8>, image::ImageError> {
    let rendered = render_annotations_with_text(base, annotations);
    let mut bytes = Vec::new();
    encode_rgba(&rendered, format, &mut bytes)?;
    Ok(bytes)
}

pub fn save_effect_composite(
    base: &RgbaImage,
    annotations: &[Annotation],
    format: ExportFormat,
    path: impl AsRef<Path>,
) -> Result<(), image::ImageError> {
    let rendered = render_annotations_with_text(base, annotations);
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    encode_rgba(&rendered, format, &mut writer)?;
    writer.flush()?;
    Ok(())
}

fn encode_rgba(
    image: &RgbaImage,
    format: ExportFormat,
    writer: &mut impl Write,
) -> Result<(), image::ImageError> {
    let (width, height) = image.dimensions();
    match format {
        ExportFormat::Png => PngEncoder::new(writer).write_image(
            image.as_raw(),
            width,
            height,
            image::ExtendedColorType::Rgba8,
        ),
        ExportFormat::Jpeg { quality } => {
            let rgb = image::DynamicImage::ImageRgba8(image.clone()).to_rgb8();
            let mut encoder = JpegEncoder::new_with_quality(writer, quality.clamp(1, 100));
            encoder.encode(rgb.as_raw(), width, height, image::ExtendedColorType::Rgb8)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, sync::atomic::{AtomicU64, Ordering}};

    use image::Rgba;

    use super::*;
    use crate::{AnnotationStyle, EffectBrushStyle, Point, TextStyle};

    static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(1);

    fn sample_image() -> RgbaImage {
        RgbaImage::from_fn(96, 48, |x, y| {
            Rgba([
                (x.saturating_mul(2)) as u8,
                (y.saturating_mul(4)) as u8,
                ((x + y).saturating_mul(2)) as u8,
                255,
            ])
        })
    }

    fn test_path(extension: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "screen-sidekick-export-test-{}-{}.{}",
            std::process::id(),
            NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed),
            extension
        ))
    }

    #[test]
    fn jpeg_quality_is_normalized() {
        assert_eq!(ExportFormat::jpeg(0), ExportFormat::Jpeg { quality: 1 });
        assert_eq!(ExportFormat::jpeg(101), ExportFormat::Jpeg { quality: 100 });
    }

    #[test]
    fn png_export_has_png_signature() {
        let bytes = encode_effect_composite(&sample_image(), &[], ExportFormat::Png).unwrap();
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn jpeg_export_has_jpeg_markers() {
        let bytes = encode_effect_composite(&sample_image(), &[], ExportFormat::jpeg(75)).unwrap();
        assert_eq!(&bytes[..2], &[0xff, 0xd8]);
        assert_eq!(&bytes[bytes.len() - 2..], &[0xff, 0xd9]);
    }

    #[test]
    fn path_based_export_loads_base_and_saves_rendered_output() {
        let source_path = test_path("png");
        let output_path = test_path("jpg");
        sample_image().save(&source_path).unwrap();

        save_annotation_export(&source_path, &[], ExportFormat::jpeg(80), &output_path).unwrap();

        let bytes = fs::read(&output_path).unwrap();
        assert_eq!(&bytes[..2], &[0xff, 0xd8]);
        assert_eq!(&bytes[bytes.len() - 2..], &[0xff, 0xd9]);

        let _ = fs::remove_file(source_path);
        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn path_based_encode_matches_in_memory_export() {
        let source_path = test_path("png");
        let base = sample_image();
        base.save(&source_path).unwrap();

        let from_path = encode_annotation_export(&source_path, &[], ExportFormat::Png).unwrap();
        let from_memory = encode_effect_composite(&base, &[], ExportFormat::Png).unwrap();
        assert_eq!(from_path, from_memory);

        let _ = fs::remove_file(source_path);
    }

    #[test]
    fn export_applies_effect_compositor_before_encoding() {
        let base = sample_image();
        let annotations = [Annotation::PixelateBrush {
            points: vec![Point { x: 40.0, y: 24.0 }],
            style: EffectBrushStyle::new(8.0, 1.0),
        }];

        let plain = encode_effect_composite(&base, &[], ExportFormat::Png).unwrap();
        let effected = encode_effect_composite(&base, &annotations, ExportFormat::Png).unwrap();
        assert_ne!(plain, effected);
    }

    #[test]
    fn export_includes_shape_annotations() {
        let base = sample_image();
        let annotations = [Annotation::Line {
            start: Point { x: 4.0, y: 4.0 },
            end: Point { x: 80.0, y: 40.0 },
            style: AnnotationStyle {
                stroke: "#ff0000".to_owned(),
                stroke_width: 2.0,
                fill: None,
            },
        }];

        let plain = encode_effect_composite(&base, &[], ExportFormat::Png).unwrap();
        let annotated = encode_effect_composite(&base, &annotations, ExportFormat::Png).unwrap();
        assert_ne!(plain, annotated);
    }

    #[test]
    fn export_includes_text_annotations() {
        let base = sample_image();
        let annotations = [Annotation::Text {
            x: 6.0,
            y: 6.0,
            text: "ไทย".to_owned(),
            style: TextStyle {
                color: "#ffffffff".to_owned(),
                font_size: 20.0,
                background: Some("#00000080".to_owned()),
            },
        }];

        let plain = encode_effect_composite(&base, &[], ExportFormat::Png).unwrap();
        let annotated = encode_effect_composite(&base, &annotations, ExportFormat::Png).unwrap();
        assert_ne!(plain, annotated);
    }
}
