use gpui::{
    App, Bounds, Context, Render, Window, WindowBackgroundAppearance, WindowBounds, WindowKind,
    WindowOptions, div, img, point, prelude::*, px, rgb, size,
};
use sidekick_core::SavedCapture;

pub struct OverlayCard {
    capture: SavedCapture,
}

impl OverlayCard {
    pub fn new(capture: SavedCapture) -> Self {
        Self { capture }
    }
}

impl Render for OverlayCard {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let image_path = self.capture.path.clone();

        div()
            .size_full()
            .p_2()
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_3()
                    .bg(rgb(0x17151f))
                    .border_1()
                    .border_color(rgb(0x353142))
                    .rounded(px(14.0))
                    .shadow_lg()
                    .child(
                        img(image_path)
                            .w_full()
                            .h(px(148.0))
                            .object_fit(gpui::ObjectFit::Contain)
                            .rounded(px(9.0)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .text_color(rgb(0xe8e5ef))
                            .text_sm()
                            .child(format!("{} × {}", self.capture.width, self.capture.height))
                            .child("Captured"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .children(["Copy", "Save", "Annotate", "Pin", "Delete"].map(|label| {
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded(px(7.0))
                                    .bg(rgb(0x292532))
                                    .text_color(rgb(0xcfc9dc))
                                    .text_xs()
                                    .child(label)
                            })),
                    ),
            )
    }
}

pub fn overlay_window_options(cx: &App) -> WindowOptions {
    let overlay_size = size(px(360.0), px(245.0));
    let margin = px(24.0);

    let bounds = cx
        .primary_display()
        .map(|display| {
            let screen = display.bounds();
            Bounds::new(
                point(
                    screen.right() - overlay_size.width - margin,
                    screen.bottom() - overlay_size.height - margin,
                ),
                overlay_size,
            )
        })
        .unwrap_or_else(|| Bounds::centered(None, overlay_size, cx));

    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        focus: false,
        show: true,
        kind: WindowKind::Floating,
        is_movable: false,
        is_resizable: false,
        is_minimizable: false,
        window_background: WindowBackgroundAppearance::Transparent,
        ..Default::default()
    }
}
