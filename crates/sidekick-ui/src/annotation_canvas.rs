use gpui::{
    App, Bounds, Context, CursorStyle, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Render, Window, WindowBounds, WindowOptions, div, img, point, prelude::*, px, rgba, size,
};
use sidekick_core::{Annotation, AnnotationStyle, EditorDocument, Point as CorePoint};

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
}

impl AnnotationCanvasView {
    pub fn new(document: EditorDocument) -> Self {
        Self {
            document,
            active_tool: AnnotationTool::Select,
            drag_start: None,
            drag_current: None,
        }
    }

    pub fn document(&self) -> &EditorDocument {
        &self.document
    }

    pub fn active_tool(&self) -> AnnotationTool {
        self.active_tool
    }

    fn set_tool(&mut self, tool: AnnotationTool) {
        self.active_tool = tool;
        self.drag_start = None;
        self.drag_current = None;
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

    fn pointer_to_base(&self, position: gpui::Point<gpui::Pixels>) -> Option<CorePoint> {
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

    fn drag_bounds(&self, geometry: ImageGeometry) -> Option<Bounds<gpui::Pixels>> {
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

    fn finish_shape(&mut self, end: CorePoint) {
        let Some(start) = self.drag_start.take() else {
            return;
        };
        self.drag_current = None;

        let x = start.x.min(end.x);
        let y = start.y.min(end.y);
        let w = (start.x - end.x).abs();
        let h = (start.y - end.y).abs();
        if w < 2.0 || h < 2.0 {
            return;
        }

        let outline = AnnotationStyle {
            stroke: "#ff3b30".to_owned(),
            stroke_width: 3.0,
            fill: None,
        };
        let filled = AnnotationStyle {
            stroke: "#ff3b30".to_owned(),
            stroke_width: 3.0,
            fill: Some("#ff3b3044".to_owned()),
        };

        let annotation = match self.active_tool {
            AnnotationTool::Rectangle => Some(Annotation::Rectangle {
                x,
                y,
                w,
                h,
                style: outline,
            }),
            AnnotationTool::FilledRectangle => Some(Annotation::Rectangle {
                x,
                y,
                w,
                h,
                style: filled,
            }),
            AnnotationTool::Ellipse => Some(Annotation::Ellipse {
                x,
                y,
                w,
                h,
                style: outline,
            }),
            AnnotationTool::Select => None,
        };

        if let Some(annotation) = annotation {
            self.document.add_annotation(annotation);
        }
    }
}

impl Render for AnnotationCanvasView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let geometry = self.image_geometry();
        let preview = self.drag_bounds(geometry);
        let annotations = self.document.annotations().to_vec();
        let base_path = self.document.base().path().to_path_buf();

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
                    )),
            )
            .child(
                div()
                    .id("annotation-canvas")
                    .relative()
                    .flex_1()
                    .w_full()
                    .cursor(match self.active_tool {
                        AnnotationTool::Select => CursorStyle::Arrow,
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
                    .children(
                        annotations
                            .into_iter()
                            .filter_map(|annotation| render_annotation(annotation, geometry)),
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
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|view, event: &MouseDownEvent, window, cx| {
                            if view.active_tool == AnnotationTool::Select {
                                return;
                            }
                            let Some(position) = view.pointer_to_base(event.position) else {
                                return;
                            };
                            view.drag_start = Some(position);
                            view.drag_current = Some(position);
                            window.refresh();
                            cx.notify();
                        }),
                    )
                    .on_mouse_move(cx.listener(|view, event: &MouseMoveEvent, window, cx| {
                        if view.drag_start.is_some()
                            && let Some(position) = view.pointer_to_base(event.position)
                        {
                            view.drag_current = Some(position);
                            window.refresh();
                            cx.notify();
                        }
                    }))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|view, event: &MouseUpEvent, window, cx| {
                            let Some(position) = view.pointer_to_base(event.position) else {
                                view.drag_start = None;
                                view.drag_current = None;
                                window.refresh();
                                cx.notify();
                                return;
                            };
                            view.finish_shape(position);
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
            view.set_tool(tool);
            window.refresh();
            cx.notify();
        }))
}

fn render_annotation(annotation: Annotation, geometry: ImageGeometry) -> Option<gpui::Div> {
    match annotation {
        Annotation::Rectangle { x, y, w, h, style } => {
            Some(styled_shape(x, y, w, h, style, geometry, false))
        }
        Annotation::Ellipse { x, y, w, h, style } => {
            Some(styled_shape(x, y, w, h, style, geometry, true))
        }
        _ => None,
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
