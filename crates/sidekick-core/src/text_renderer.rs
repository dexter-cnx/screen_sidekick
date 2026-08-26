use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache, Wrap};
use image::{Rgba, RgbaImage};

use crate::{Annotation, TextStyle, annotation_renderer::render_annotations};

const TEXT_BACKGROUND_PADDING_X: f32 = 4.0;
const TEXT_BACKGROUND_PADDING_Y: f32 = 2.0;
const LINE_HEIGHT_SCALE: f32 = 1.25;

pub fn render_annotations_with_text(base: &RgbaImage, annotations: &[Annotation]) -> RgbaImage {
    if !annotations
        .iter()
        .any(|annotation| matches!(annotation, Annotation::Text { .. }))
    {
        return render_annotations(base, annotations);
    }

    let mut output = base.clone();
    let mut font_system = FontSystem::new();
    let mut cache = SwashCache::new();
    let mut chunk_start = 0;

    for (index, annotation) in annotations.iter().enumerate() {
        let Annotation::Text { x, y, text, style } = annotation else {
            continue;
        };

        if chunk_start < index {
            output = render_annotations(&output, &annotations[chunk_start..index]);
        }
        render_text_annotation(
            &mut output,
            *x,
            *y,
            text,
            style,
            &mut font_system,
            &mut cache,
        );
        chunk_start = index + 1;
    }

    if chunk_start < annotations.len() {
        output = render_annotations(&output, &annotations[chunk_start..]);
    }

    output
}

fn render_text_annotation(
    image: &mut RgbaImage,
    x: f32,
    y: f32,
    text: &str,
    style: &TextStyle,
    font_system: &mut FontSystem,
    cache: &mut SwashCache,
) {
    if text.is_empty() {
        return;
    }

    let font_size = style.font_size.clamp(1.0, 512.0);
    let line_height = font_size * LINE_HEIGHT_SCALE;
    let metrics = Metrics::new(font_size, line_height);
    let mut buffer = Buffer::new(font_system, metrics);
    buffer.set_size(None, None);
    buffer.set_wrap(Wrap::None);
    buffer.set_text(text, &Attrs::new(), Shaping::Advanced, None);
    buffer.shape_until_scroll(font_system, false);

    let (text_width, text_height) = buffer.layout_runs().fold((0.0_f32, 0.0_f32), |size, run| {
        (
            size.0.max(run.line_w),
            size.1.max(run.line_top + run.line_height),
        )
    });

    if let Some(background) = style.background.as_deref() {
        let background = parse_color(background);
        fill_rect(
            image,
            x - TEXT_BACKGROUND_PADDING_X,
            y - TEXT_BACKGROUND_PADDING_Y,
            x + text_width + TEXT_BACKGROUND_PADDING_X,
            y + text_height + TEXT_BACKGROUND_PADDING_Y,
            background,
        );
    }

    let foreground = parse_color(&style.color);
    let style_alpha = foreground[3];
    let draw_color = Color::rgb(foreground[0], foreground[1], foreground[2]);
    let origin_x = x.round() as i32;
    let origin_y = y.round() as i32;

    buffer.draw(
        font_system,
        cache,
        draw_color,
        |gx, gy, width, height, color| {
            let callback_rgba = color.as_rgba();
            let alpha = ((u16::from(callback_rgba[3]) * u16::from(style_alpha) + 127) / 255) as u8;
            let pixel = Rgba([callback_rgba[0], callback_rgba[1], callback_rgba[2], alpha]);
            fill_rect_i32(image, origin_x + gx, origin_y + gy, width, height, pixel);
        },
    );
}

fn fill_rect(image: &mut RgbaImage, left: f32, top: f32, right: f32, bottom: f32, color: Rgba<u8>) {
    let min_x = left.floor().max(0.0) as u32;
    let min_y = top.floor().max(0.0) as u32;
    let max_x = right.ceil().max(0.0).min(image.width() as f32) as u32;
    let max_y = bottom.ceil().max(0.0).min(image.height() as f32) as u32;
    for py in min_y..max_y {
        for px in min_x..max_x {
            blend_pixel(image, px, py, color);
        }
    }
}

fn fill_rect_i32(image: &mut RgbaImage, x: i32, y: i32, width: u32, height: u32, color: Rgba<u8>) {
    for dy in 0..height {
        for dx in 0..width {
            let px = x + dx as i32;
            let py = y + dy as i32;
            if px >= 0 && py >= 0 && px < image.width() as i32 && py < image.height() as i32 {
                blend_pixel(image, px as u32, py as u32, color);
            }
        }
    }
}

fn parse_color(value: &str) -> Rgba<u8> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    let bytes = hex.as_bytes();
    if !matches!(bytes.len(), 6 | 8) || !bytes.iter().all(u8::is_ascii_hexdigit) {
        return Rgba([255, 255, 255, 255]);
    }

    let red = parse_hex_byte(bytes[0], bytes[1]);
    let green = parse_hex_byte(bytes[2], bytes[3]);
    let blue = parse_hex_byte(bytes[4], bytes[5]);
    let alpha = if bytes.len() == 8 {
        parse_hex_byte(bytes[6], bytes[7])
    } else {
        255
    };
    Rgba([red, green, blue, alpha])
}

fn parse_hex_byte(high: u8, low: u8) -> u8 {
    (hex_nibble(high) << 4) | hex_nibble(low)
}

fn hex_nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        b'A'..=b'F' => value - b'A' + 10,
        _ => 0,
    }
}

fn blend_pixel(image: &mut RgbaImage, x: u32, y: u32, source: Rgba<u8>) {
    let source_alpha = source[3] as f32 / 255.0;
    if source_alpha <= 0.0 {
        return;
    }
    if source_alpha >= 1.0 {
        image.put_pixel(x, y, source);
        return;
    }

    let destination = *image.get_pixel(x, y);
    let destination_alpha = destination[3] as f32 / 255.0;
    let output_alpha = source_alpha + destination_alpha * (1.0 - source_alpha);
    if output_alpha <= f32::EPSILON {
        image.put_pixel(x, y, Rgba([0, 0, 0, 0]));
        return;
    }

    let blend_channel = |source_channel: u8, destination_channel: u8| {
        ((source_channel as f32 * source_alpha
            + destination_channel as f32 * destination_alpha * (1.0 - source_alpha))
            / output_alpha)
            .round() as u8
    };
    image.put_pixel(
        x,
        y,
        Rgba([
            blend_channel(source[0], destination[0]),
            blend_channel(source[1], destination[1]),
            blend_channel(source[2], destination[2]),
            (output_alpha * 255.0).round() as u8,
        ]),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AnnotationStyle, EffectBrushStyle, Point};

    #[test]
    fn unicode_text_changes_rendered_pixels() {
        let base = RgbaImage::from_pixel(240, 80, Rgba([0, 0, 0, 255]));
        let text = Annotation::Text {
            x: 10.0,
            y: 10.0,
            text: "สวัสดี Screen Sidekick".to_owned(),
            style: TextStyle {
                color: "#ffffffff".to_owned(),
                font_size: 24.0,
                background: None,
            },
        };

        let rendered = render_annotations_with_text(&base, &[text]);
        assert_ne!(rendered, base);
    }

    #[test]
    fn text_background_is_rasterized() {
        let base = RgbaImage::from_pixel(160, 60, Rgba([0, 0, 0, 0]));
        let text = Annotation::Text {
            x: 20.0,
            y: 10.0,
            text: "A".to_owned(),
            style: TextStyle {
                color: "#ffffffff".to_owned(),
                font_size: 20.0,
                background: Some("#ff000080".to_owned()),
            },
        };

        let rendered = render_annotations_with_text(&base, &[text]);
        assert!(rendered.pixels().any(|pixel| pixel[3] > 0));
    }

    #[test]
    fn text_keeps_annotation_order_around_effects() {
        let base = RgbaImage::from_pixel(180, 80, Rgba([0, 0, 0, 255]));
        let text = Annotation::Text {
            x: 20.0,
            y: 20.0,
            text: "Order".to_owned(),
            style: TextStyle {
                color: "#ffffffff".to_owned(),
                font_size: 24.0,
                background: None,
            },
        };
        let blur = Annotation::BlurBrush {
            points: vec![Point { x: 45.0, y: 30.0 }],
            style: EffectBrushStyle::new(12.0, 1.0),
        };
        let line = Annotation::Line {
            start: Point { x: 10.0, y: 30.0 },
            end: Point { x: 100.0, y: 30.0 },
            style: AnnotationStyle {
                stroke: "#ff0000ff".to_owned(),
                stroke_width: 3.0,
                fill: None,
            },
        };

        let text_then_blur =
            render_annotations_with_text(&base, &[text.clone(), blur.clone(), line.clone()]);
        let blur_then_text = render_annotations_with_text(&base, &[blur, text, line]);
        assert_ne!(text_then_blur, blur_then_text);
    }
}
