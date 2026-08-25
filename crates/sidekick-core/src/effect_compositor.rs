use image::{Rgba, RgbaImage};

use crate::{Annotation, EffectBrushStyle, Point};

pub fn composite_effect_brushes(base: &RgbaImage, annotations: &[Annotation]) -> RgbaImage {
    let mut output = base.clone();

    for annotation in annotations {
        match annotation {
            Annotation::BlurBrush { points, style } => {
                apply_blur_brush(&mut output, points, style);
            }
            Annotation::PixelateBrush { points, style } => {
                apply_pixelate_brush(&mut output, points, style);
            }
            _ => {}
        }
    }

    output
}

fn apply_blur_brush(image: &mut RgbaImage, points: &[Point], style: &EffectBrushStyle) {
    if points.is_empty() || style.strength <= 0.0 {
        return;
    }

    let source = image.clone();
    let sigma = (style.radius * style.strength * 0.35).max(0.5);
    let blurred = image::imageops::blur(&source, sigma);
    apply_masked_pixels(image, &blurred, points, style.radius);
}

fn apply_pixelate_brush(image: &mut RgbaImage, points: &[Point], style: &EffectBrushStyle) {
    if points.is_empty() || style.strength <= 0.0 {
        return;
    }

    let source = image.clone();
    let block_size = (2.0 + style.strength * style.radius * 0.5)
        .round()
        .clamp(2.0, 64.0) as u32;
    let pixelated = pixelate(&source, block_size);
    apply_masked_pixels(image, &pixelated, points, style.radius);
}

fn apply_masked_pixels(
    destination: &mut RgbaImage,
    effect: &RgbaImage,
    points: &[Point],
    radius: f32,
) {
    if destination.dimensions() != effect.dimensions() || points.is_empty() {
        return;
    }

    let radius = radius.max(1.0);
    let (width, height) = destination.dimensions();
    for y in 0..height {
        for x in 0..width {
            let sample = Point {
                x: x as f32 + 0.5,
                y: y as f32 + 0.5,
            };
            if point_is_inside_brush(sample, points, radius) {
                destination.put_pixel(x, y, *effect.get_pixel(x, y));
            }
        }
    }
}

fn point_is_inside_brush(sample: Point, points: &[Point], radius: f32) -> bool {
    if points.len() == 1 {
        return distance_squared(sample, points[0]) <= radius * radius;
    }

    points
        .windows(2)
        .any(|segment| point_segment_distance_squared(sample, segment[0], segment[1]) <= radius * radius)
}

fn point_segment_distance_squared(point: Point, start: Point, end: Point) -> f32 {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length_squared = dx * dx + dy * dy;
    if length_squared <= f32::EPSILON {
        return distance_squared(point, start);
    }

    let projection = ((point.x - start.x) * dx + (point.y - start.y) * dy) / length_squared;
    let t = projection.clamp(0.0, 1.0);
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

fn pixelate(source: &RgbaImage, block_size: u32) -> RgbaImage {
    let block_size = block_size.max(1);
    let (width, height) = source.dimensions();
    let mut output = source.clone();

    for block_y in (0..height).step_by(block_size as usize) {
        for block_x in (0..width).step_by(block_size as usize) {
            let end_x = (block_x + block_size).min(width);
            let end_y = (block_y + block_size).min(height);
            let mut sums = [0_u64; 4];
            let mut count = 0_u64;

            for y in block_y..end_y {
                for x in block_x..end_x {
                    let pixel = source.get_pixel(x, y).0;
                    for (sum, channel) in sums.iter_mut().zip(pixel) {
                        *sum += u64::from(channel);
                    }
                    count += 1;
                }
            }

            if count == 0 {
                continue;
            }

            let average = Rgba([
                (sums[0] / count) as u8,
                (sums[1] / count) as u8,
                (sums[2] / count) as u8,
                (sums[3] / count) as u8,
            ]);
            for y in block_y..end_y {
                for x in block_x..end_x {
                    output.put_pixel(x, y, average);
                }
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(width: u32, height: u32) -> RgbaImage {
        RgbaImage::from_fn(width, height, |x, y| {
            Rgba([
                (x * 20).min(255) as u8,
                (y * 20).min(255) as u8,
                ((x + y) * 10).min(255) as u8,
                255,
            ])
        })
    }

    #[test]
    fn non_effect_annotations_leave_pixels_unchanged() {
        let base = gradient(8, 8);
        let annotation = Annotation::Rectangle {
            x: 1.0,
            y: 1.0,
            w: 3.0,
            h: 3.0,
            style: crate::AnnotationStyle {
                stroke: "#ffffff".to_owned(),
                stroke_width: 1.0,
                fill: None,
            },
        };

        assert_eq!(composite_effect_brushes(&base, &[annotation]), base);
    }

    #[test]
    fn pixelate_changes_only_pixels_inside_brush_mask() {
        let base = gradient(12, 12);
        let annotation = Annotation::PixelateBrush {
            points: vec![Point { x: 5.0, y: 5.0 }],
            style: EffectBrushStyle::new(2.0, 1.0),
        };
        let output = composite_effect_brushes(&base, &[annotation]);

        assert_ne!(output.get_pixel(5, 5), base.get_pixel(5, 5));
        assert_eq!(output.get_pixel(0, 0), base.get_pixel(0, 0));
        assert_eq!(output.get_pixel(11, 11), base.get_pixel(11, 11));
    }

    #[test]
    fn blur_changes_brushed_region_without_mutating_base() {
        let mut base = RgbaImage::from_pixel(9, 9, Rgba([0, 0, 0, 255]));
        base.put_pixel(4, 4, Rgba([255, 255, 255, 255]));
        let original = base.clone();
        let annotation = Annotation::BlurBrush {
            points: vec![Point { x: 4.5, y: 4.5 }],
            style: EffectBrushStyle::new(2.5, 1.0),
        };

        let output = composite_effect_brushes(&base, &[annotation]);

        assert_eq!(base, original);
        assert_ne!(output.get_pixel(4, 4), base.get_pixel(4, 4));
        assert_eq!(output.get_pixel(0, 0), base.get_pixel(0, 0));
    }

    #[test]
    fn brush_mask_covers_segment_between_sampled_points() {
        let points = [Point { x: 2.0, y: 2.0 }, Point { x: 8.0, y: 2.0 }];
        assert!(point_is_inside_brush(Point { x: 5.0, y: 2.5 }, &points, 1.0));
        assert!(!point_is_inside_brush(Point { x: 5.0, y: 4.0 }, &points, 1.0));
    }
}
