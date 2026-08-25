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

    let Some(mask) = BrushMask::rasterize(points, style.radius, image.width(), image.height()) else {
        return;
    };
    let sigma = (style.radius * style.strength * 0.35).max(0.5);
    let kernel_padding = (sigma * 3.0).ceil() as u32;
    let source_bounds = mask.bounds.expanded(kernel_padding, image.width(), image.height());
    let source = image::imageops::crop_imm(
        image,
        source_bounds.x,
        source_bounds.y,
        source_bounds.width,
        source_bounds.height,
    )
    .to_image();
    let blurred = image::imageops::blur(&source, sigma);
    apply_masked_roi(image, &blurred, source_bounds, &mask);
}

fn apply_pixelate_brush(image: &mut RgbaImage, points: &[Point], style: &EffectBrushStyle) {
    if points.is_empty() || style.strength <= 0.0 {
        return;
    }

    let Some(mask) = BrushMask::rasterize(points, style.radius, image.width(), image.height()) else {
        return;
    };
    let block_size = (2.0 + style.strength * style.radius * 0.5)
        .round()
        .clamp(2.0, 64.0) as u32;
    let source_bounds = mask
        .bounds
        .expanded(block_size, image.width(), image.height());
    let source = image::imageops::crop_imm(
        image,
        source_bounds.x,
        source_bounds.y,
        source_bounds.width,
        source_bounds.height,
    )
    .to_image();
    let pixelated = pixelate(&source, block_size);
    apply_masked_roi(image, &pixelated, source_bounds, &mask);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PixelBounds {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl PixelBounds {
    fn expanded(self, padding: u32, image_width: u32, image_height: u32) -> Self {
        let x = self.x.saturating_sub(padding);
        let y = self.y.saturating_sub(padding);
        let right = self
            .x
            .saturating_add(self.width)
            .saturating_add(padding)
            .min(image_width);
        let bottom = self
            .y
            .saturating_add(self.height)
            .saturating_add(padding)
            .min(image_height);
        Self {
            x,
            y,
            width: right.saturating_sub(x),
            height: bottom.saturating_sub(y),
        }
    }
}

#[derive(Debug, Clone)]
struct BrushMask {
    bounds: PixelBounds,
    pixels: Vec<bool>,
}

impl BrushMask {
    fn rasterize(
        points: &[Point],
        radius: f32,
        image_width: u32,
        image_height: u32,
    ) -> Option<Self> {
        if points.is_empty() || image_width == 0 || image_height == 0 {
            return None;
        }

        let radius = radius.max(1.0);
        let min_x = points
            .iter()
            .map(|point| point.x - radius)
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as u32;
        let min_y = points
            .iter()
            .map(|point| point.y - radius)
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as u32;
        let max_x = points
            .iter()
            .map(|point| point.x + radius)
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .max(0.0) as u32;
        let max_y = points
            .iter()
            .map(|point| point.y + radius)
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .max(0.0) as u32;

        let right = max_x.min(image_width);
        let bottom = max_y.min(image_height);
        if min_x >= right || min_y >= bottom {
            return None;
        }

        let bounds = PixelBounds {
            x: min_x,
            y: min_y,
            width: right - min_x,
            height: bottom - min_y,
        };
        let mut mask = Self {
            bounds,
            pixels: vec![false; bounds.width as usize * bounds.height as usize],
        };

        if points.len() == 1 {
            mask.rasterize_segment(points[0], points[0], radius);
        } else {
            for segment in points.windows(2) {
                mask.rasterize_segment(segment[0], segment[1], radius);
            }
        }
        Some(mask)
    }

    fn rasterize_segment(&mut self, start: Point, end: Point, radius: f32) {
        let radius_squared = radius * radius;
        let min_x = ((start.x.min(end.x) - radius).floor().max(self.bounds.x as f32) as u32)
            .max(self.bounds.x);
        let min_y = ((start.y.min(end.y) - radius).floor().max(self.bounds.y as f32) as u32)
            .max(self.bounds.y);
        let max_x = ((start.x.max(end.x) + radius).ceil() as u32)
            .min(self.bounds.x + self.bounds.width);
        let max_y = ((start.y.max(end.y) + radius).ceil() as u32)
            .min(self.bounds.y + self.bounds.height);

        for y in min_y..max_y {
            for x in min_x..max_x {
                let sample = Point {
                    x: x as f32 + 0.5,
                    y: y as f32 + 0.5,
                };
                if point_segment_distance_squared(sample, start, end) <= radius_squared {
                    let local_x = x - self.bounds.x;
                    let local_y = y - self.bounds.y;
                    let index = local_y as usize * self.bounds.width as usize + local_x as usize;
                    self.pixels[index] = true;
                }
            }
        }
    }

    fn contains(&self, x: u32, y: u32) -> bool {
        if x < self.bounds.x
            || y < self.bounds.y
            || x >= self.bounds.x + self.bounds.width
            || y >= self.bounds.y + self.bounds.height
        {
            return false;
        }
        let local_x = x - self.bounds.x;
        let local_y = y - self.bounds.y;
        self.pixels[local_y as usize * self.bounds.width as usize + local_x as usize]
    }
}

fn apply_masked_roi(
    destination: &mut RgbaImage,
    effect: &RgbaImage,
    effect_bounds: PixelBounds,
    mask: &BrushMask,
) {
    if effect.dimensions() != (effect_bounds.width, effect_bounds.height) {
        return;
    }

    for y in mask.bounds.y..mask.bounds.y + mask.bounds.height {
        for x in mask.bounds.x..mask.bounds.x + mask.bounds.width {
            if mask.contains(x, y) {
                let effect_x = x - effect_bounds.x;
                let effect_y = y - effect_bounds.y;
                destination.put_pixel(x, y, *effect.get_pixel(effect_x, effect_y));
            }
        }
    }
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
        let mask = BrushMask::rasterize(&points, 1.0, 20, 20).expect("mask");

        assert!(mask.contains(5, 2));
        assert!(!mask.contains(5, 4));
    }

    #[test]
    fn brush_mask_uses_compact_stroke_bounds() {
        let points = [Point { x: 100.0, y: 80.0 }, Point { x: 120.0, y: 90.0 }];
        let mask = BrushMask::rasterize(&points, 4.0, 3840, 2160).expect("mask");

        assert!(mask.bounds.width <= 28);
        assert!(mask.bounds.height <= 18);
        assert_eq!(
            mask.pixels.len(),
            mask.bounds.width as usize * mask.bounds.height as usize
        );
    }
}
