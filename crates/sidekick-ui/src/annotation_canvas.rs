use gpui::{
    App, Bounds, Context, CursorStyle, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Pixels, Point, Render, Window, WindowBounds, WindowOptions, div, point, prelude::*, px, rgba,
    size,
};
use sidekick_core::{Annotation, AnnotationStyle, EditorDocument};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationTool {
    Select,
    Rectangle,
    FilledRectangle,
    Ellipse,
}

pub struct AnnotationCanvasView {
    document: EditorDocument,
    active_tool: AnnotationTool,
    drag_start: Option<Point<Pixels>>,
    drag_current: Option<Point<Pixels>>,
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

    fn drag_bounds(&self) -> Option<Bounds<Pixels>> {
        let start = self.drag_start?;
        let current = self.drag_current?;
        let left = start.x.min(current.x);
        let top = start.y.min(current.y);
        let width = (start.x - current.x).abs();
        let height = (start.y - current.y).abs();
        Some(Bounds::new(point(left, top), size(width, height)))
    }

    fn finish_shape(&mut self, end: Point<Pixels>) {
        let Some(start) = self.drag_start.take() else {
            return;
        };
        self.drag_current = None;

        let x = f32::from(start.x.min(end.x));
        let y = f32::from(start.y.min(end.y));
        let w = f32::from((start.x - end.x).abs());
        let h = f32::from((start.y - end.y).abs());
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
            fill: Some("#44ff3b30".to_owned()),
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
        let preview = self.drag_bounds();
        let annotations = self.document.annotations().to_vec();

        div()
            .size_full()
            .bg(rgba(0x17171aff))
            .child(
                div()
                    .h(px(48.0))
                    .w_full()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .bg(rgba(0x242428ff))
                    .child(tool_button("Select", AnnotationTool::Select, self.active_tool, cx))
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
                    .child(tool_button("Ellipse", AnnotationTool::Ellipse, self.active_tool, cx)),
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
                    .children(annotations.into_iter().filter_map(render_annotation))
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
                            view.drag_start = Some(event.position);
                            view.drag_current = Some(event.position);
                            window.refresh();
                            cx.notify();
                        }),
                    )
                    .on_mouse_move(cx.listener(|view, event: &MouseMoveEvent, window, cx| {
                        if view.drag_start.is_some() {
                            view.drag_current = Some(event.position);
                            window.refresh();
                            cx.notify();
                        }
                    }))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|view, event: &MouseUpEvent, window, cx| {
                            view.finish_shape(event.position);
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

fn render_annotation(annotation: Annotation) -> Option<gpui::Div> {
    match annotation {
        Annotation::Rectangle { x, y, w, h, style } => Some(
            div()
                .absolute()
                .left(px(x))
                .top(px(y))
                .w(px(w))
                .h(px(h))
                .border_2()
                .border_color(rgba(0xff3b30ff))
                .when(style.fill.is_some(), |shape| shape.bg(rgba(0xff3b3033))),
        ),
        Annotation::Ellipse { x, y, w, h, .. } => Some(
            div()
                .absolute()
                .left(px(x))
                .top(px(y))
                .w(px(w))
                .h(px(h))
                .rounded(px(w.min(h) / 2.0))
                .border_2()
                .border_color(rgba(0xff3b30ff)),
        ),
        _ => None,
    }
}

pub fn annotation_window_options(cx: &App) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(1100.0), px(760.0)),
            cx,
        ))),
        focus: true,
        show: true,
        ..Default::default()
    }
}
