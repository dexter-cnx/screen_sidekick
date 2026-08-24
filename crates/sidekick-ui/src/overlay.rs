use gpui::{
    App, Bounds, ClipboardItem, Context, Image, ImageFormat, Render, Window,
    WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions, div, img, point,
    prelude::*, px, rgb, size,
};
use sidekick_core::SavedCapture;
use std::{path::PathBuf, sync::mpsc::Sender};

const OVERLAY_WIDTH: f32 = 360.0;
const OVERLAY_HEIGHT: f32 = 245.0;
const OVERLAY_MARGIN: f32 = 24.0;
const STACK_OFFSET: f32 = 28.0;
const PEEK_WIDTH: f32 = 104.0;
const PEEK_HEIGHT: f32 = 40.0;

pub struct OverlayCard {
    capture: SavedCapture,
    stack_size: usize,
    delete_sender: Sender<PathBuf>,
}

impl OverlayCard {
    pub fn new(capture: SavedCapture, stack_size: usize, delete_sender: Sender<PathBuf>) -> Self {
        Self {
            capture,
            stack_size,
            delete_sender,
        }
    }
}

impl Render for OverlayCard {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let image_path = self.capture.path.clone();
        let copy_path = image_path.clone();
        let delete_path = image_path.clone();
        let delete_sender = self.delete_sender.clone();
        let stack_label = if self.stack_size == 1 {
            "Latest · 1 capture".to_owned()
        } else {
            format!("Latest · {} captures", self.stack_size)
        };

        div().size_full().p_2().child(
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
                        .child(stack_label),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .id("copy-capture")
                                .px_2()
                                .py_1()
                                .rounded(px(7.0))
                                .bg(rgb(0x292532))
                                .text_color(rgb(0xcfc9dc))
                                .text_xs()
                                .cursor_pointer()
                                .child("Copy")
                                .on_click(move |_, window, cx| match std::fs::read(&copy_path) {
                                    Ok(bytes) => {
                                        let image = Image::from_bytes(ImageFormat::Png, bytes);
                                        cx.write_to_clipboard(ClipboardItem::new_image(&image));
                                        window.refresh();
                                    }
                                    Err(error) => {
                                        eprintln!("Screen Sidekick copy failed: {error}");
                                    }
                                }),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded(px(7.0))
                                .bg(rgb(0x292532))
                                .text_color(rgb(0x7f7989))
                                .text_xs()
                                .child("Saved"),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded(px(7.0))
                                .bg(rgb(0x292532))
                                .text_color(rgb(0x7f7989))
                                .text_xs()
                                .child("Annotate"),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded(px(7.0))
                                .bg(rgb(0x292532))
                                .text_color(rgb(0x7f7989))
                                .text_xs()
                                .child("Pin"),
                        )
                        .child(
                            div()
                                .id("delete-capture")
                                .px_2()
                                .py_1()
                                .rounded(px(7.0))
                                .bg(rgb(0x292532))
                                .text_color(rgb(0xcfc9dc))
                                .text_xs()
                                .cursor_pointer()
                                .child("Delete")
                                .on_click(move |_, window, _cx| {
                                    match std::fs::remove_file(&delete_path) {
                                        Ok(()) => {
                                            let _ = delete_sender.send(delete_path.clone());
                                            window.remove_window();
                                        }
                                        Err(error)
                                            if error.kind() == std::io::ErrorKind::NotFound =>
                                        {
                                            let _ = delete_sender.send(delete_path.clone());
                                            window.remove_window();
                                        }
                                        Err(error) => {
                                            eprintln!("Screen Sidekick delete failed: {error}");
                                        }
                                    }
                                }),
                        ),
                ),
        )
    }
}

pub struct PeekTab {
    stack_size: usize,
    activate_sender: Sender<()>,
}

impl PeekTab {
    pub fn new(stack_size: usize, activate_sender: Sender<()>) -> Self {
        Self {
            stack_size,
            activate_sender,
        }
    }
}

impl Render for PeekTab {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let activate_sender = self.activate_sender.clone();
        div()
            .id("peek-tab")
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(12.0))
            .bg(rgb(0x17151f))
            .border_1()
            .border_color(rgb(0x353142))
            .shadow_lg()
            .text_color(rgb(0xe8e5ef))
            .text_xs()
            .cursor_pointer()
            .child(format!("Screens · {}", self.stack_size))
            .on_click(move |_, window, _cx| {
                let _ = activate_sender.send(());
                window.remove_window();
            })
    }
}

pub fn overlay_window_options(cx: &App, stack_slot: usize) -> WindowOptions {
    let overlay_size = size(px(OVERLAY_WIDTH), px(OVERLAY_HEIGHT));
    let margin = px(OVERLAY_MARGIN);
    let stack_offset = px(STACK_OFFSET * stack_slot as f32);

    let bounds = cx
        .primary_display()
        .map(|display| {
            let screen = display.bounds();
            Bounds::new(
                point(
                    screen.right() - overlay_size.width - margin,
                    screen.bottom() - overlay_size.height - margin - stack_offset,
                ),
                overlay_size,
            )
        })
        .unwrap_or_else(|| Bounds::centered(None, overlay_size, cx));

    floating_window_options(bounds)
}

pub fn peek_window_options(cx: &App) -> WindowOptions {
    let peek_size = size(px(PEEK_WIDTH), px(PEEK_HEIGHT));
    let margin = px(OVERLAY_MARGIN);
    let bounds = cx
        .primary_display()
        .map(|display| {
            let screen = display.bounds();
            Bounds::new(
                point(
                    screen.right() - peek_size.width - margin,
                    screen.bottom() - peek_size.height - margin,
                ),
                peek_size,
            )
        })
        .unwrap_or_else(|| Bounds::centered(None, peek_size, cx));

    floating_window_options(bounds)
}

fn floating_window_options(bounds: Bounds<gpui::Pixels>) -> WindowOptions {
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
