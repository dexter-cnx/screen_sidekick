use image::{Rgba, RgbaImage};

use crate::{Annotation, AnnotationStyle, MarkerStyle, Point, composite_effect_brushes};

pub fn render_annotations(base: &RgbaImage, annotations: &[Annotation]) -> RgbaImage {
    let mut output = base.clone();

    for annotation in annotations {
        match annotation {
            Annotation::BlurBrush { .. } | Annotation::PixelateBrush { .. } => {
                output = composite_effect_brushes(&output, std::slice::from_ref(annotation));
            }
            Annotation::Rectangle { x, y, w, h, style } => {
                draw_rectangle(&mut output, *x, *y, *w, *h, style);
            }
            Annotation::Ellipse { x, y, w, h, style } => {
                draw_ellipse(&mut output, *x, *y, *w, *h, style);
            }
            Annotation::Line { start, end, style } => {
                draw_segment(
                    &mut output,
                    *start,
                    *end,
                    style.stroke_width,
                    parse_color(&style.stroke),
                );
            }
            Annotation::Arrow { start, end, style } => {
                draw_arrow(&mut output, *start, *end, style);
            }
            Annotation::Freehand { points, style } => {
                draw_freehand(&mut output, points, style);
            }
            Annotation::NumberMarker {
                x,
                y,
                number,
                style,
            } => {
                draw_number_marker(&mut output, *x, *y, *number, style);
            }
            Annotation::Text { .. } | Annotation::HighlightDimmer { .. } => {}
        }
    }

    output
}

fn draw_rectangle(image: &mut RgbaImage, x: f32, y: f32, w: f32, h: f32, style: &AnnotationStyle) {
    let left = x.min(x + w);
    let right = x.max(x + w);
    let top = y.min(y + h);
    let bottom = y.max(y + h);

    if let Some(fill) = style.fill.as_deref().map(parse_color) {
        fill_rect(image, left, top, right, bottom, fill);
    }

    let stroke = parse_color(&style.stroke);
    let width = style.stroke_width.max(1.0);
    draw_segment(
        image,
        Point { x: left, y: top },
        Point { x: right, y: top },
        width,
        stroke,
    );
    draw_segment(
        image,
        Point { x: right, y: top },
        Point {
            x: right,
            y: bottom,
        },
        width,
        stroke,
    );
    draw_segment(
        image,
        Point {
            x: right,
            y: bottom,
        },
        Point { x: left, y: bottom },
        width,
        stroke,
    );
    draw_segment(
        image,
        Point { x: left, y: bottom },
        Point { x: left, y: top },
        width,
        stroke,
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

fn draw_ellipse(image: &mut RgbaImage, x: f32, y: f32, w: f32, h: f32, style: &AnnotationStyle) {
    let left = x.min(x + w);
    let right = x.max(x + w);
    let top = y.min(y + h);
    let bottom = y.max(y + h);
    let rx = ((right - left) * 0.5).max(0.5);
    let ry = ((bottom - top) * 0.5).max(0.5);
    let cx = (left + right) * 0.5;
    let cy = (top + bottom) * 0.5;
    let stroke_width = style.stroke_width.max(1.0);
    let inner_rx = (rx - stroke_width).max(0.0);
    let inner_ry = (ry - stroke_width).max(0.0);
    let stroke = parse_color(&style.stroke);
    let fill = style.fill.as_deref().map(parse_color);

    let min_x = left.floor().max(0.0) as u32;
    let min_y = top.floor().max(0.0) as u32;
    let max_x = right.ceil().max(0.0).min(image.width() as f32) as u32;
    let max_y = bottom.ceil().max(0.0).min(image.height() as f32) as u32;

    for py in min_y..max_y {
        for px in min_x..max_x {
            let dx = px as f32 + 0.5 - cx;
            let dy = py as f32 + 0.5 - cy;
            let outer = dx * dx / (rx * rx) + dy * dy / (ry * ry) <= 1.0;
            if !outer {
                continue;
            }
            let inner = inner_rx > 0.0
                && inner_ry > 0.0
                && dx * dx / (inner_rx * inner_rx) + dy * dy / (inner_ry * inner_ry) <= 1.0;
            if !inner {
                blend_pixel(image, px, py, stroke);
            } else if let Some(fill) = fill {
                blend_pixel(image, px, py, fill);
            }
        }
    }
}

fn draw_arrow(image: &mut RgbaImage, start: Point, end: Point, style: &AnnotationStyle) {
    let color = parse_color(&style.stroke);
    let width = style.stroke_width.max(1.0);
    draw_segment(image, start, end, width, color);

    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length = (dx * dx + dy * dy).sqrt();
    if length <= f32::EPSILON {
        return;
    }
    let ux = dx / length;
    let uy = dy / length;
    let head = (width * 4.0).max(10.0).min(length * 0.5);
    let wing = head * 0.55;
    let base = Point {
        x: end.x - ux * head,
        y: end.y - uy * head,
    };
    let perpendicular = Point { x: -uy, y: ux };
    let left = Point {
        x: base.x + perpendicular.x * wing,
        y: base.y + perpendicular.y * wing,
    };
    let right = Point {
        x: base.x - perpendicular.x * wing,
        y: base.y - perpendicular.y * wing,
    };
    draw_segment(image, end, left, width, color);
    draw_segment(image, end, right, width, color);
}

fn draw_freehand(image: &mut RgbaImage, points: &[Point], style: &AnnotationStyle) {
    let color = parse_color(&style.stroke);
    let width = style.stroke_width.max(1.0);
    match points {
        [] => {}
        [point] => draw_disk(image, *point, width * 0.5, color),
        _ => {
            for segment in points.windows(2) {
                draw_segment(image, segment[0], segment[1], width, color);
            }
        }
    }
}

fn draw_segment(image: &mut RgbaImage, start: Point, end: Point, width: f32, color: Rgba<u8>) {
    let radius = (width.max(1.0) * 0.5).max(0.5);
    let min_x = (start.x.min(end.x) - radius).floor().max(0.0) as u32;
    let min_y = (start.y.min(end.y) - radius).floor().max(0.0) as u32;
    let max_x = (start.x.max(end.x) + radius)
        .ceil()
        .max(0.0)
        .min(image.width() as f32) as u32;
    let max_y = (start.y.max(end.y) + radius)
        .ceil()
        .max(0.0)
        .min(image.height() as f32) as u32;
    let radius_squared = radius * radius;

    for py in min_y..max_y {
        for px in min_x..max_x {
            let sample = Point {
                x: px as f32 + 0.5,
                y: py as f32 + 0.5,
            };
            if point_segment_distance_squared(sample, start, end) <= radius_squared {
                blend_pixel(image, px, py, color);
            }
        }
    }
}

fn draw_disk(image: &mut RgbaImage, center: Point, radius: f32, color: Rgba<u8>) {
    draw_segment(image, center, center, radius * 2.0, color);
}

fn draw_number_marker(image: &mut RgbaImage, x: f32, y: f32, number: u32, style: &MarkerStyle) {
    let radius = (style.diameter.max(4.0) * 0.5).max(2.0);
    draw_disk(
        image,
        Point { x, y },
        radius,
        parse_color(&style.background),
    );

    let text = number.to_string();
    let scale = (style.diameter / 10.0).floor().max(1.0) as u32;
    let glyph_width = 3 * scale;
    let gap = scale;
    let total_width = text.len() as u32 * glyph_width + text.len().saturating_sub(1) as u32 * gap;
    let start_x = x.round() as i32 - total_width as i32 / 2;
    let start_y = y.round() as i32 - (5 * scale) as i32 / 2;
    let foreground = parse_color(&style.foreground);

    for (index, digit) in text.chars().enumerate() {
        if let Some(pattern) = digit_pattern(digit) {
            draw_digit(
                image,
                start_x + index as i32 * (glyph_width + gap) as i32,
                start_y,
                scale,
                pattern,
                foreground,
            );
        }
    }
}

fn draw_digit(
    image: &mut RgbaImage,
    x: i32,
    y: i32,
    scale: u32,
    pattern: [u8; 5],
    color: Rgba<u8>,
) {
    for (row, bits) in pattern.into_iter().enumerate() {
        for col in 0..3 {
            if bits & (1 << (2 - col)) == 0 {
                continue;
            }
            for sy in 0..scale {
                for sx in 0..scale {
                    let px = x + (col * scale + sx) as i32;
                    let py = y + (row as u32 * scale + sy) as i32;
                    if px >= 0 && py >= 0 && px < image.width() as i32 && py < image.height() as i32
                    {
                        blend_pixel(image, px as u32, py as u32, color);
                    }
                }
            }
        }
    }
}

fn digit_pattern(digit: char) -> Option<[u8; 5]> {
    Some(match digit {
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        _ => return None,
    })
}

fn point_segment_distance_squared(point: Point, start: Point, end: Point) -> f32 {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length_squared = dx * dx + dy * dy;
    if length_squared <= f32::EPSILON {
        return distance_squared(point, start);
    }
    let t =
        (((point.x - start.x) * dx + (point.y - start.y) * dy) / length_squared).clamp(0.0, 1.0);
    let closest = Point {
        x: start.x + t * dx,
        y: start.y + t * dy,
    };
    distance_squared(point, closest)
}

fn distance_squared(a: Point, b: Point) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
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
    use crate::{AnnotationStyle, EffectBrushStyle};

    fn style() -> AnnotationStyle {
        AnnotationStyle {
            stroke: "#ff0000".to_owned(),
            stroke_width: 2.0,
            fill: Some("#00ff0080".to_owned()),
        }
    }

    #[test]
    fn rectangle_rasterization_changes_only_its_region() {
        let base = RgbaImage::from_pixel(20, 20, Rgba([0, 0, 0, 255]));
        let annotation = Annotation::Rectangle {
            x: 4.0,
            y: 5.0,
            w: 8.0,
            h: 6.0,
            style: style(),
        };
        let rendered = render_annotations(&base, &[annotation]);

        assert_ne!(rendered.get_pixel(4, 5), base.get_pixel(4, 5));
        assert_eq!(rendered.get_pixel(0, 0), base.get_pixel(0, 0));
    }

    #[test]
    fn freehand_rasterizes_between_sampled_points() {
        let base = RgbaImage::from_pixel(20, 20, Rgba([0, 0, 0, 255]));
        let annotation = Annotation::Freehand {
            points: vec![Point { x: 2.0, y: 10.0 }, Point { x: 16.0, y: 10.0 }],
            style: style(),
        };
        let rendered = render_annotations(&base, &[annotation]);

        assert_ne!(rendered.get_pixel(8, 10), base.get_pixel(8, 10));
    }

    #[test]
    fn annotation_order_is_preserved_across_effects() {
        let base = RgbaImage::from_pixel(24, 24, Rgba([0, 0, 0, 255]));
        let line = Annotation::Line {
            start: Point { x: 4.0, y: 12.0 },
            end: Point { x: 20.0, y: 12.0 },
            style: AnnotationStyle {
                stroke: "#ffffff".to_owned(),
                stroke_width: 2.0,
                fill: None,
            },
        };
        let blur = Annotation::BlurBrush {
            points: vec![Point { x: 12.0, y: 12.0 }],
            style: EffectBrushStyle::new(4.0, 1.0),
        };

        let line_then_blur = render_annotations(&base, &[line.clone(), blur.clone()]);
        let blur_then_line = render_annotations(&base, &[blur, line]);
        assert_ne!(line_then_blur, blur_then_line);
    }

    #[test]
    fn marker_rasterizes_background_and_digits() {
        let base = RgbaImage::from_pixel(32, 32, Rgba([0, 0, 0, 255]));
        let marker = Annotation::NumberMarker {
            x: 16.0,
            y: 16.0,
            number: 12,
            style: MarkerStyle {
                foreground: "#ffffff".to_owned(),
                background: "#ff0000".to_owned(),
                diameter: 20.0,
            },
        };
        let rendered = render_annotations(&base, &[marker]);

        assert_ne!(rendered.get_pixel(16, 16), base.get_pixel(16, 16));
        assert_eq!(rendered.get_pixel(0, 0), base.get_pixel(0, 0));
    }

    #[test]
    fn eight_digit_colors_use_rgba_order() {
        assert_eq!(parse_color("#01020380"), Rgba([1, 2, 3, 128]));
    }

    #[test]
    fn malformed_utf8_color_falls_back_without_panicking() {
        assert_eq!(parse_color("#aéaaa"), Rgba([255, 255, 255, 255]));
    }

    #[test]
    fn source_over_preserves_transparent_destination_alpha() {
        let mut image = RgbaImage::from_pixel(1, 1, Rgba([10, 20, 30, 0]));
        blend_pixel(&mut image, 0, 0, Rgba([200, 100, 50, 128]));
        assert_eq!(image.get_pixel(0, 0), &Rgba([200, 100, 50, 128]));
    }

    #[test]
    fn source_over_combines_partial_alpha() {
        let mut image = RgbaImage::from_pixel(1, 1, Rgba([0, 0, 255, 128]));
        blend_pixel(&mut image, 0, 0, Rgba([255, 0, 0, 128]));
        assert_eq!(image.get_pixel(0, 0)[3], 192);
    }
}
