use gpui::{
    App, Bounds, Context, Render, SharedString, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use sidekick_core::CaptureWindow;
use std::sync::mpsc::Sender;

const CHOOSER_WIDTH: f32 = 560.0;
const CHOOSER_HEIGHT: f32 = 440.0;

pub struct WindowChooserView {
    windows: Vec<CaptureWindow>,
    selection_sender: Sender<u32>,
}

impl WindowChooserView {
    pub fn new(windows: Vec<CaptureWindow>, selection_sender: Sender<u32>) -> Self {
        Self {
            windows,
            selection_sender,
        }
    }
}

impl Render for WindowChooserView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let rows = self.windows.iter().map(|capture_window| {
            let id = capture_window.id;
            let sender = self.selection_sender.clone();
            let title = if capture_window.title.trim().is_empty() {
                "Untitled window".to_owned()
            } else {
                capture_window.title.clone()
            };
            let app_name = capture_window.app_name.clone();
            let details = format!(
                "{} × {}{}",
                capture_window.width,
                capture_window.height,
                if capture_window.is_focused {
                    " · Focused"
                } else {
                    ""
                }
            );

            div()
                .id(SharedString::from(format!("capture-window-{id}")))
                .flex()
                .items_center()
                .justify_between()
                .gap_4()
                .px_4()
                .py_3()
                .rounded(px(10.0))
                .bg(rgb(0x211e29))
                .border_1()
                .border_color(rgb(0x353142))
                .cursor_pointer()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .min_w_0()
                        .child(div().text_sm().text_color(rgb(0xe8e5ef)).child(title))
                        .child(div().text_xs().text_color(rgb(0x9f98aa)).child(app_name)),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .text_xs()
                        .text_color(rgb(0x817a8c))
                        .child(details),
                )
                .on_click(move |_, window, _cx| {
                    let _ = sender.send(id);
                    window.remove_window();
                })
        });

        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .bg(rgb(0x15131b))
            .text_color(rgb(0xe8e5ef))
            .child(div().text_xl().child("Choose a Window"))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xa9a3b4))
                    .child("Select a visible window to capture."),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .overflow_scroll()
                    .child(div().flex().flex_col().gap_2().children(rows)),
            )
    }
}

pub fn window_chooser_options(cx: &App) -> WindowOptions {
    let window_size = size(px(CHOOSER_WIDTH), px(CHOOSER_HEIGHT));
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            window_size,
            cx,
        ))),
        titlebar: Some(TitlebarOptions {
            title: Some("Choose Window".into()),
            ..Default::default()
        }),
        focus: true,
        is_movable: true,
        is_resizable: false,
        ..Default::default()
    }
}
