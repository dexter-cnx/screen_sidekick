use gpui::{
    App, Bounds, Context, CursorStyle, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Pixels, Point, Render, Window, WindowBackgroundAppearance, WindowBounds, WindowKind,
    WindowOptions, div, point, prelude::*, px, rgba, size,
};
use sidekick_core::CaptureRegion;
use std::sync::mpsc::Sender;

pub struct AreaSelectorView {
    start: Option<Point<Pixels>>,
    current: Option<Point<Pixels>>,
    selection_sender: Sender<CaptureRegion>,
}

impl AreaSelectorView {
    pub fn new(selection_sender: Sender<CaptureRegion>) -> Self {
        Self {
            start: None,
            current: None,
            selection_sender,
        }
    }

    fn selection_bounds(&self) -> Option<Bounds<Pixels>> {
        let start = self.start?;
        let current = self.current?;
        let left = start.x.min(current.x);
        let top = start.y.min(current.y);
        let width = (start.x - current.x).abs();
        let height = (start.y - current.y).abs();
        Some(Bounds::new(point(left, top), size(width, height)))
    }
}

impl Render for AreaSelectorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selection = self.selection_bounds();

        div()
            .id("area-selector")
            .size_full()
            .relative()
            .cursor(CursorStyle::Crosshair)
            .bg(rgba(0x00000066))
            .child(
                div()
                    .absolute()
                    .top(px(20.0))
                    .left(px(20.0))
                    .px_3()
                    .py_2()
                    .rounded(px(8.0))
                    .bg(rgba(0x15131bdd))
                    .text_color(gpui::white())
                    .text_sm()
                    .child("Drag to select an area"),
            )
            .when_some(selection, |root, bounds| {
                root.child(
                    div()
                        .absolute()
                        .left(bounds.origin.x)
                        .top(bounds.origin.y)
                        .w(bounds.size.width)
                        .h(bounds.size.height)
                        .border_2()
                        .border_color(gpui::white())
                        .bg(rgba(0xffffff18)),
                )
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, event: &MouseDownEvent, window, cx| {
                    view.start = Some(event.position);
                    view.current = Some(event.position);
                    window.refresh();
                    cx.notify();
                }),
            )
            .on_mouse_move(cx.listener(|view, event: &MouseMoveEvent, window, cx| {
                if view.start.is_some() {
                    view.current = Some(event.position);
                    window.refresh();
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|view, event: &MouseUpEvent, window, _cx| {
                    let Some(start) = view.start.take() else {
                        return;
                    };
                    view.current = None;

                    let left = start.x.min(event.position.x).max(px(0.0));
                    let top = start.y.min(event.position.y).max(px(0.0));
                    let width = (start.x - event.position.x).abs();
                    let height = (start.y - event.position.y).abs();
                    let scale_factor = window.scale_factor();

                    let region = CaptureRegion::new(
                        (f32::from(left) * scale_factor).round() as i32,
                        (f32::from(top) * scale_factor).round() as i32,
                        (f32::from(width) * scale_factor).round() as u32,
                        (f32::from(height) * scale_factor).round() as u32,
                    );

                    if let Ok(region) = region {
                        let _ = view.selection_sender.send(region);
                        window.remove_window();
                    } else {
                        window.refresh();
                    }
                }),
            )
    }
}

pub fn area_selector_window_options(cx: &App) -> WindowOptions {
    let bounds = cx
        .primary_display()
        .map(|display| display.bounds())
        .unwrap_or_else(|| Bounds::centered(None, size(px(1280.0), px(800.0)), cx));

    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        focus: true,
        show: true,
        kind: WindowKind::Floating,
        is_movable: false,
        is_resizable: false,
        is_minimizable: false,
        window_background: WindowBackgroundAppearance::Transparent,
        ..Default::default()
    }
}
