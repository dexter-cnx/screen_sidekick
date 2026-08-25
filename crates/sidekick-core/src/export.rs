use std::{fs::File, io::BufWriter, path::Path};

use image::{ImageEncoder, RgbaImage, codecs::jpeg::JpegEncoder, codecs::png::PngEncoder};

use crate::{Annotation, composite_effect_brushes};

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

pub fn encode_effect_composite(
    base: &RgbaImage,
    annotations: &[Annotation],
    format: ExportFormat,
) -> Result<Vec<u8>, image::ImageError> {
    let rendered = composite_effect_brushes(base, annotations);
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
    let rendered = composite_effect_brushes(base, annotations);
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    encode_rgba(&rendered, format, &mut writer)
}

fn encode_rgba(
    image: &RgbaImage,
    format: ExportFormat,
    writer: &mut impl std::io::Write,
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
            let mut encoder = JpegEncoder::new_with_quality(writer, quality.clamp(1, 100));
            encoder.encode(
                image.as_raw(),
                width,
                height,
                image::ExtendedColorType::Rgba8,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use image::Rgba;

    use super::*;
    use crate::{EffectBrushStyle, Point};

    fn sample_image() -> RgbaImage {
        RgbaImage::from_fn(8, 8, |x, y| {
            Rgba([(x * 28) as u8, (y * 28) as u8, ((x + y) * 14) as u8, 255])
        })
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
    fn export_applies_effect_compositor_before_encoding() {
        let base = sample_image();
        let annotations = [Annotation::PixelateBrush {
            points: vec![Point { x: 4.0, y: 4.0 }],
            style: EffectBrushStyle::new(3.0, 1.0),
        }];

        let plain = encode_effect_composite(&base, &[], ExportFormat::Png).unwrap();
        let effected = encode_effect_composite(&base, &annotations, ExportFormat::Png).unwrap();
        assert_ne!(plain, effected);
    }
}
