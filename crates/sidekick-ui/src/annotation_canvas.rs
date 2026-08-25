use gpui::{
    App, Bounds, Context, CursorStyle, Entity, Focusable, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, PathBuilder, Pixels, Point, Render, Window, WindowBounds,
    WindowOptions, canvas, div, img, point, prelude::*, px, rgba, size,
};
use sidekick_core::{
    Annotation, AnnotationStyle, EditorDocument, MarkerStyle, Point as CorePoint, TextStyle,
};

use crate::{text_annotation::TextAnnotationDraft, text_input::TextDraftInput};

const WINDOW_WIDTH: f32 = 1100.0;
const WINDOW_HEIGHT: f32 = 760.0;
const TOOLBAR_HEIGHT: f32 = 48.0;
const CANVAS_WIDTH: f32 = WINDOW_WIDTH;
const CANVAS_HEIGHT: f32 = WINDOW_HEIGHT - TOOLBAR_HEIGHT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationTool {
    Select,
    Rectangle,
    FilledRectangle,
    Ellipse,
    Line,
    Arrow,
    Freehand,
    NumberMarker,
    Text,
}

#[derive(Debug, Clone, Copy)]
struct ImageGeometry {
    origin_x: f32,
    origin_y: f32,
    width: f32,
    height: f32,
    scale: f32,
}

pub struct AnnotationCanvasView {
    document: EditorDocument,
    active_tool: AnnotationTool,
    drag_start: Option<CorePoint>,
    drag_current: Option<CorePoint>,
    freehand_points: Vec<CorePoint>,
    text_draft: Option<TextAnnotationDraft>,
    text_input: Option<Entity<TextDraftInput>>,
}

impl AnnotationCanvasView {
    pub fn new(document: EditorDocument) -> Self {
        Self {
            document,
            active_tool: AnnotationTool::Select,
            drag_start: None,
            drag_current: None,
            freehand_points: Vec::new(),
            text_draft: None,
            text_input: None,
        }
    }

    pub fn document(&self) -> &EditorDocument {
        &self.document
    }

    pub fn active_tool(&self) -> AnnotationTool {
        self.active_tool
    }

    fn set_tool(&mut self, tool: AnnotationTool, cx: &mut Context<Self>) {
        if self.active_tool == AnnotationTool::Text
            && tool != AnnotationTool::Text
            && !self.commit_text_draft(cx)
        {
            return;
        }
        self.active_tool = tool;
        self.drag_start = None;
        self.drag_current = None;
        self.freehand_points.clear();
    }

    fn image_geometry(&self) -> ImageGeometry {
        let base_width = self.document.base().width() as f32;
        let base_height = self.document.base().height() as f32;
        let scale = (CANVAS_WIDTH / base_width)
            .min(CANVAS_HEIGHT / base_height)
            .max(f32::EPSILON);
        let width = base_width * scale;
        let height = base_height * scale;

        ImageGeometry {
            origin_x: (CANVAS_WIDTH - width) / 2.0,
            origin_y: (CANVAS_HEIGHT - height) / 2.0,
            width,
            height,
            scale,
        }
    }

    fn pointer_to_base(&self, position: Point<Pixels>) -> Option<CorePoint> {
        let geometry = self.image_geometry();
        let canvas_x = f32::from(position.x);
        let canvas_y = f32::from(position.y) - TOOLBAR_HEIGHT;
        let image_x = canvas_x - geometry.origin_x;
        let image_y = canvas_y - geometry.origin_y;

        if image_x < 0.0 || image_y < 0.0 || image_x > geometry.width || image_y > geometry.height {
            return None;
        }

        Some(CorePoint {
            x: image_x / geometry.scale,
            y: image_y / geometry.scale,
        })
    }

    fn drag_bounds(&self, geometry: ImageGeometry) -> Option<Bounds<Pixels>> {
        if matches!(
            self.active_tool,
            AnnotationTool::Line
                | AnnotationTool::Arrow
                | AnnotationTool::Freehand
                | AnnotationTool::NumberMarker
                | AnnotationTool::Text
        ) {
            return None;
        }

        let start = self.drag_start?;
        let current = self.drag_current?;
        let left = start.x.min(current.x) * geometry.scale + geometry.origin_x;
        let top = start.y.min(current.y) * geometry.scale + geometry.origin_y;
        let width = (start.x - current.x).abs() * geometry.scale;
        let height = (start.y - current.y).abs() * geometry.scale;
        Some(Bounds::new(
            point(px(left), px(top)),
            size(px(width), px(height)),
        ))
    }

    fn outline_style() -> AnnotationStyle {
        AnnotationStyle {
            stroke: "#ff3b30".to_owned(),
            stroke_width: 3.0,
            fill: None,
        }
    }

    fn marker_style() -> MarkerStyle {
        MarkerStyle {
            foreground: "#ffffffff".to_owned(),
            background: "#ff3b30ff".to_owned(),
            diameter: 28.0,
        }
    }

    fn add_number_marker(&mut self, position: CorePoint) {
        let number = next_marker_number(self.document.annotations());
        self.document.add_annotation(Annotation::NumberMarker {
            x: position.x,
            y: position.y,
            number,
            style: Self::marker_style(),
        });
    }

    fn start_text_draft(
        &mut self,
        position: CorePoint,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.commit_text_draft(cx) {
            if let Some(input) = self.text_input.as_ref() {
                window.focus(&input.focus_handle(cx), cx);
            }
            return;
        }
        self.text_draft = Some(TextAnnotationDraft::new(position));

        if let Some(input) = self.text_input.as_ref().cloned() {
            input.update(cx, |input, cx| input.reset(cx));
            window.focus(&input.focus_handle(cx), cx);
        }
    }

    fn commit_text_draft(&mut self, cx: &mut Context<Self>) -> bool {
        if self.text_draft.is_none() {
            return true;
        }
        let Some(input) = self.text_input.as_ref().cloned() else {
            return false;
        };
        if input.read(cx).is_composing() {
            return false;
        }

        let text = input.read(cx).content().to_owned();
        let Some(draft) = self.text_draft.take() else {
            return true;
        };
        if let Some(annotation) = draft.commit(text) {
            self.document.add_annotation(annotation);
        }
        input.update(cx, |input, cx| input.reset(cx));
        true
    }

    fn finish_drag(&mut self, end: CorePoint) {
        let Some(start) = self.drag_start.take() else {
            return;
        };
        self.drag_current = None;

        let annotation = match self.active_tool {
            AnnotationTool::Rectangle
            | AnnotationTool::FilledRectangle
            | AnnotationTool::Ellipse => {
                let x = start.x.min(end.x);
                let y = start.y.min(end.y);
                let w = (start.x - end.x).abs();
                let h = (start.y - end.y).abs();
                if w < 2.0 || h < 2.0 {
                    return;
                }

                let mut style = Self::outline_style();
                if self.active_tool == AnnotationTool::FilledRectangle {
                    style.fill = Some("#ff3b3044".to_owned());
                }

                match self.active_tool {
                    AnnotationTool::Rectangle | AnnotationTool::FilledRectangle => {
                        Some(Annotation::Rectangle { x, y, w, h, style })
                    }
                    AnnotationTool::Ellipse => Some(Annotation::Ellipse { x, y, w, h, style }),
                    _ => None,
                }
            }
            AnnotationTool::Line => {
                if distance(start, end) < 2.0 {
                    return;
                }
                Some(Annotation::Line {
                    start,
                    end,
                    style: Self::outline_style(),
                })
            }
            AnnotationTool::Arrow => {
                if distance(start, end) < 2.0 {
                    return;
                }
                Some(Annotation::Arrow {
                    start,
                    end,
                    style: Self::outline_style(),
                })
            }
            AnnotationTool::Select
            | AnnotationTool::Freehand
            | AnnotationTool::NumberMarker
            | AnnotationTool::Text => None,
        };

        if let Some(annotation) = annotation {
            self.document.add_annotation(annotation);
        }
    }

    fn finish_freehand(&mut self) {
        if self.freehand_points.len() < 2 {
            self.freehand_points.clear();
            return;
        }

        let points = std::mem::take(&mut self.freehand_points);
        self.document.add_annotation(Annotation::Freehand {
            points,
            style: Self::outline_style(),
        });
    }
}

impl Render for AnnotationCanvasView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.text_input.is_none() {
            self.text_input = Some(cx.new(TextDraftInput::new));
        }

        let geometry = self.image_geometry();
        let preview = self.drag_bounds(geometry);
        let annotation_layers = self
            .document
            .annotations()
            .iter()
            .cloned()
            .filter_map(|annotation| render_annotation_layer(annotation, geometry));
        let base_path = self.document.base().path().to_path_buf();
        let active_tool = self.active_tool;
        let drag_start = self.drag_start;
        let drag_current = self.drag_current;
        let freehand_preview = self.freehand_points.clone();
        let text_editor = self.text_draft.as_ref().and_then(|draft| {
            self.text_input
                .as_ref()
                .cloned()
                .map(|input| (draft.position(), input))
        });
        let has_text_draft = self.text_draft.is_some();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgba(0x17171aff))
            .child(
                div()
                    .h(px(TOOLBAR_HEIGHT))
                    .w_full()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .bg(rgba(0x242428ff))
                    .child(tool_button(
                        "Select",
                        AnnotationTool::Select,
                        self.active_tool,
                        cx,
                    ))
                    .child(tool_button(
                        "Rectangle",
                        AnnotationTool::Rectangle,
                        self.active_tool,
                        cx,
                    ))
                    .child(tool_button(
                        "Filled",
                        AnnotationTool::FilledRectangle,
                        self.active_tool,
                        cx,
                    ))
                    .child(tool_button(
                        "Ellipse",
                        AnnotationTool::Ellipse,
                        self.active_tool,
                        cx,
                    ))
                    .child(tool_button(
                        "Line",
                        AnnotationTool::Line,
                        self.active_tool,
                        cx,
                    ))
                    .child(tool_button(
                        "Arrow",
                        AnnotationTool::Arrow,
                        self.active_tool,
                        cx,
                    ))
                    .child(tool_button(
                        "Freehand",
                        AnnotationTool::Freehand,
                        self.active_tool,
                        cx,
                    ))
                    .child(tool_button(
                        "Marker",
                        AnnotationTool::NumberMarker,
                        self.active_tool,
                        cx,
                    ))
                    .child(tool_button(
                        "Text",
                        AnnotationTool::Text,
                        self.active_tool,
                        cx,
                    ))
                    .when(has_text_draft, |toolbar| {
                        toolbar.child(text_done_button(cx))
                    }),
            )
            .child(
                div()
                    .id("annotation-canvas")
                    .relative()
                    .flex_1()
                    .w_full()
                    .cursor(match self.active_tool {
                        AnnotationTool::Select => CursorStyle::Arrow,
                        AnnotationTool::Text => CursorStyle::IBeam,
                        _ => CursorStyle::Crosshair,
                    })
                    .bg(rgba(0x2a2a2eff))
                    .child(
                        img(base_path)
                            .absolute()
                            .left(px(geometry.origin_x))
                            .top(px(geometry.origin_y))
                            .w(px(geometry.width))
                            .h(px(geometry.height))
                            .object_fit(gpui::ObjectFit::Contain),
                    )
                    .children(annotation_layers)
                    .child(
                        canvas(
                            move |_, _, _| {},
                            move |bounds, _, window, _| {
                                let canvas_origin = bounds.origin;

                                if matches!(
                                    active_tool,
                                    AnnotationTool::Line | AnnotationTool::Arrow
                                ) && let (Some(start), Some(end)) = (drag_start, drag_current)
                                {
                                    let annotation = if active_tool == AnnotationTool::Arrow {
                                        Annotation::Arrow {
                                            start,
                                            end,
                                            style: AnnotationCanvasView::outline_style(),
                                        }
                                    } else {
                                        Annotation::Line {
                                            start,
                                            end,
                                            style: AnnotationCanvasView::outline_style(),
                                        }
                                    };
                                    paint_path_annotation(
                                        window,
                                        annotation,
                                        geometry,
                                        canvas_origin,
                                    );
                                }

                                if active_tool == AnnotationTool::Freehand
                                    && freehand_preview.len() >= 2
                                {
                                    paint_polyline(
                                        window,
                                        &freehand_preview,
                                        &AnnotationCanvasView::outline_style(),
                                        geometry,
                                        canvas_origin,
                                    );
                                }
                            },
                        )
                        .absolute()
                        .left(px(0.0))
                        .top(px(0.0))
                        .size_full(),
                    )
                    .when_some(preview, |canvas, bounds| {
                        canvas.child(
                            div()
                                .absolute()
                                .left(bounds.origin.x)
                                .top(bounds.origin.y)
                                .w(bounds.size.width)
                                .h(bounds.size.height)
                                .border_2()
                                .border_color(rgba(0xff3b30ff))
                                .bg(rgba(0xff3b3022)),
                        )
                    })
                    .when_some(text_editor, |canvas, (position, input)| {
                        let left = geometry.origin_x + position.x * geometry.scale;
                        let top = geometry.origin_y + position.y * geometry.scale;
                        canvas.child(
                            div()
                                .absolute()
                                .left(px(left))
                                .top(px(top))
                                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                                .child(input),
                        )
                    })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|view, event: &MouseDownEvent, window, cx| {
                            if view.active_tool == AnnotationTool::Select {
                                return;
                            }
                            let Some(position) = view.pointer_to_base(event.position) else {
                                return;
                            };

                            match view.active_tool {
                                AnnotationTool::Freehand => {
                                    view.freehand_points.clear();
                                    view.freehand_points.push(position);
                                }
                                AnnotationTool::NumberMarker => {
                                    view.add_number_marker(position);
                                }
                                AnnotationTool::Text => {
                                    view.start_text_draft(position, window, cx);
                                }
                                _ => {
                                    view.drag_start = Some(position);
                                    view.drag_current = Some(position);
                                }
                            }
                            window.refresh();
                            cx.notify();
                        }),
                    )
                    .on_mouse_move(cx.listener(|view, event: &MouseMoveEvent, window, cx| {
                        let Some(position) = view.pointer_to_base(event.position) else {
                            return;
                        };

                        if view.active_tool == AnnotationTool::Freehand {
                            if !view.freehand_points.is_empty()
                                && view
                                    .freehand_points
                                    .last()
                                    .is_none_or(|last| distance(*last, position) >= 0.5)
                            {
                                view.freehand_points.push(position);
                                window.refresh();
                                cx.notify();
                            }
                        } else if view.drag_start.is_some() {
                            view.drag_current = Some(position);
                            window.refresh();
                            cx.notify();
                        }
                    }))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|view, event: &MouseUpEvent, window, cx| {
                            if matches!(
                                view.active_tool,
                                AnnotationTool::NumberMarker | AnnotationTool::Text
                            ) {
                                return;
                            }
                            if view.active_tool == AnnotationTool::Freehand {
                                if let Some(position) = view.pointer_to_base(event.position)
                                    && view
                                        .freehand_points
                                        .last()
                                        .is_none_or(|last| distance(*last, position) >= 0.5)
                                {
                                    view.freehand_points.push(position);
                                }
                                view.finish_freehand();
                            } else if let Some(position) = view.pointer_to_base(event.position) {
                                view.finish_drag(position);
                            } else {
                                view.drag_start = None;
                                view.drag_current = None;
                            }
                            window.refresh();
                            cx.notify();
                        }),
                    ),
            )
    }
}

fn tool_button(
    label: &'static str,
    tool: AnnotationTool,
    active_tool: AnnotationTool,
    cx: &mut Context<AnnotationCanvasView>,
) -> impl IntoElement {
    let active = tool == active_tool;
    div()
        .id(label)
        .px_3()
        .py_1()
        .rounded(px(6.0))
        .cursor_pointer()
        .bg(if active {
            rgba(0x4a4a52ff)
        } else {
            rgba(0x303036ff)
        })
        .text_color(gpui::white())
        .child(label)
        .on_click(cx.listener(move |view, _, window, cx| {
            view.set_tool(tool, cx);
            window.refresh();
            cx.notify();
        }))
}

fn text_done_button(cx: &mut Context<AnnotationCanvasView>) -> impl IntoElement {
    div()
        .id("text-done")
        .px_3()
        .py_1()
        .rounded(px(6.0))
        .cursor_pointer()
        .bg(rgba(0x2f7d4aff))
        .text_color(gpui::white())
        .child("Done")
        .on_click(cx.listener(|view, _, window, cx| {
            view.commit_text_draft(cx);
            window.refresh();
            cx.notify();
        }))
}

fn render_annotation_layer(
    annotation: Annotation,
    geometry: ImageGeometry,
) -> Option<gpui::AnyElement> {
    match annotation {
        Annotation::Rectangle { x, y, w, h, style } => {
            Some(styled_shape(x, y, w, h, style, geometry, false).into_any_element())
        }
        Annotation::Ellipse { x, y, w, h, style } => {
            Some(styled_shape(x, y, w, h, style, geometry, true).into_any_element())
        }
        Annotation::NumberMarker {
            x,
            y,
            number,
            style,
        } => Some(render_number_marker(x, y, number, style, geometry).into_any_element()),
        Annotation::Text { x, y, text, style } => {
            Some(render_text_annotation(x, y, text, style, geometry).into_any_element())
        }
        annotation @ (Annotation::Line { .. }
        | Annotation::Arrow { .. }
        | Annotation::Freehand { .. }) => Some(
            canvas(
                move |_, _, _| {},
                move |bounds, _, window, _| {
                    paint_path_annotation(window, annotation, geometry, bounds.origin);
                },
            )
            .absolute()
            .left(px(0.0))
            .top(px(0.0))
            .size_full()
            .into_any_element(),
        ),
    }
}

fn styled_shape(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    style: AnnotationStyle,
    geometry: ImageGeometry,
    ellipse: bool,
) -> gpui::Div {
    let left = geometry.origin_x + x * geometry.scale;
    let top = geometry.origin_y + y * geometry.scale;
    let width = w * geometry.scale;
    let height = h * geometry.scale;
    let stroke = parse_color(&style.stroke).unwrap_or_else(|| rgba(0xff3b30ff));
    let fill = style
        .fill
        .as_deref()
        .and_then(parse_color)
        .unwrap_or_else(|| rgba(0x00000000));
    let border_width = (style.stroke_width * geometry.scale).max(1.0);

    let shape = div()
        .absolute()
        .left(px(left))
        .top(px(top))
        .w(px(width))
        .h(px(height))
        .border_color(stroke)
        .bg(fill);
    let shape = if border_width < 1.5 {
        shape.border_1()
    } else if border_width < 2.5 {
        shape.border_2()
    } else if border_width < 3.5 {
        shape.border_3()
    } else {
        shape.border_4()
    };

    if ellipse {
        shape.rounded(px(width.min(height) / 2.0))
    } else {
        shape
    }
}

fn render_number_marker(
    x: f32,
    y: f32,
    number: u32,
    style: MarkerStyle,
    geometry: ImageGeometry,
) -> gpui::Div {
    let diameter = style.diameter * geometry.scale;
    let left = geometry.origin_x + x * geometry.scale - diameter / 2.0;
    let top = geometry.origin_y + y * geometry.scale - diameter / 2.0;
    let foreground = parse_color(&style.foreground).unwrap_or_else(|| rgba(0xffffffff));
    let background = parse_color(&style.background).unwrap_or_else(|| rgba(0xff3b30ff));

    div()
        .absolute()
        .left(px(left))
        .top(px(top))
        .w(px(diameter))
        .h(px(diameter))
        .rounded(px(diameter / 2.0))
        .flex()
        .items_center()
        .justify_center()
        .bg(background)
        .text_color(foreground)
        .text_size(px(diameter * 0.52))
        .child(number.to_string())
}

fn render_text_annotation(
    x: f32,
    y: f32,
    text: String,
    style: TextStyle,
    geometry: ImageGeometry,
) -> gpui::Div {
    let left = geometry.origin_x + x * geometry.scale;
    let top = geometry.origin_y + y * geometry.scale;
    let color = parse_color(&style.color).unwrap_or_else(|| rgba(0xffffffff));
    let background = style.background.as_deref().and_then(parse_color);
    let font_size = (style.font_size * geometry.scale).max(8.0);

    let layer = div()
        .absolute()
        .left(px(left))
        .top(px(top))
        .text_color(color)
        .text_size(px(font_size))
        .child(text);

    if let Some(background) = background {
        layer.bg(background).px_1().rounded(px(3.0))
    } else {
        layer
    }
}

fn next_marker_number(annotations: &[Annotation]) -> u32 {
    annotations
        .iter()
        .filter_map(|annotation| match annotation {
            Annotation::NumberMarker { number, .. } => Some(*number),
            _ => None,
        })
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

fn paint_path_annotation(
    window: &mut Window,
    annotation: Annotation,
    geometry: ImageGeometry,
    canvas_origin: Point<Pixels>,
) {
    match annotation {
        Annotation::Line { start, end, style } => {
            paint_line(window, start, end, &style, geometry, canvas_origin);
        }
        Annotation::Arrow { start, end, style } => {
            paint_line(window, start, end, &style, geometry, canvas_origin);
            paint_arrow_head(window, start, end, &style, geometry, canvas_origin);
        }
        Annotation::Freehand { points, style } => {
            paint_polyline(window, &points, &style, geometry, canvas_origin);
        }
        _ => {}
    }
}

fn paint_polyline(
    window: &mut Window,
    points: &[CorePoint],
    style: &AnnotationStyle,
    geometry: ImageGeometry,
    canvas_origin: Point<Pixels>,
) {
    if points.len() < 2 {
        return;
    }

    let line_width = px((style.stroke_width * geometry.scale).max(1.0));
    let mut builder = PathBuilder::stroke(line_width);
    for (index, point_value) in points.iter().copied().enumerate() {
        let point_value = to_window_point(point_value, geometry, canvas_origin);
        if index == 0 {
            builder.move_to(point_value);
        } else {
            builder.line_to(point_value);
        }
    }

    if let Ok(path) = builder.build() {
        window.paint_path(
            path,
            parse_color(&style.stroke).unwrap_or_else(|| rgba(0xff3b30ff)),
        );
    }
}

fn paint_line(
    window: &mut Window,
    start: CorePoint,
    end: CorePoint,
    style: &AnnotationStyle,
    geometry: ImageGeometry,
    canvas_origin: Point<Pixels>,
) {
    paint_polyline(window, &[start, end], style, geometry, canvas_origin);
}

fn paint_arrow_head(
    window: &mut Window,
    start: CorePoint,
    end: CorePoint,
    style: &AnnotationStyle,
    geometry: ImageGeometry,
    canvas_origin: Point<Pixels>,
) {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length = (dx * dx + dy * dy).sqrt();
    if length <= f32::EPSILON {
        return;
    }

    let ux = dx / length;
    let uy = dy / length;
    let head_length = 14.0_f32.min(length * 0.4).max(6.0);
    let wing = head_length * 0.55;
    let base_x = end.x - ux * head_length;
    let base_y = end.y - uy * head_length;
    let left = CorePoint {
        x: base_x - uy * wing,
        y: base_y + ux * wing,
    };
    let right = CorePoint {
        x: base_x + uy * wing,
        y: base_y - ux * wing,
    };

    paint_polyline(window, &[left, end, right], style, geometry, canvas_origin);
}

fn to_window_point(
    point_value: CorePoint,
    geometry: ImageGeometry,
    canvas_origin: Point<Pixels>,
) -> Point<Pixels> {
    point(
        canvas_origin.x + px(geometry.origin_x + point_value.x * geometry.scale),
        canvas_origin.y + px(geometry.origin_y + point_value.y * geometry.scale),
    )
}

fn distance(a: CorePoint, b: CorePoint) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    (dx * dx + dy * dy).sqrt()
}

fn parse_color(value: &str) -> Option<gpui::Rgba> {
    let hex = value.strip_prefix('#')?;
    let packed = match hex.len() {
        6 => (u32::from_str_radix(hex, 16).ok()? << 8) | 0xff,
        8 => u32::from_str_radix(hex, 16).ok()?,
        _ => return None,
    };
    Some(rgba(packed))
}

pub fn annotation_window_options(cx: &App) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT)),
            cx,
        ))),
        focus: true,
        show: true,
        is_resizable: false,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_numbers_continue_after_highest_existing_marker() {
        let annotations = vec![
            Annotation::NumberMarker {
                x: 1.0,
                y: 2.0,
                number: 2,
                style: AnnotationCanvasView::marker_style(),
            },
            Annotation::Rectangle {
                x: 0.0,
                y: 0.0,
                w: 10.0,
                h: 10.0,
                style: AnnotationCanvasView::outline_style(),
            },
            Annotation::NumberMarker {
                x: 3.0,
                y: 4.0,
                number: 7,
                style: AnnotationCanvasView::marker_style(),
            },
        ];

        assert_eq!(next_marker_number(&annotations), 8);
        assert_eq!(next_marker_number(&[]), 1);
    }
}
