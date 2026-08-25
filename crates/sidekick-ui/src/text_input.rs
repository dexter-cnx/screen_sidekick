use std::ops::Range;

use gpui::{
    App, Bounds, Context, CursorStyle, Element, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, FocusHandle, Focusable, GlobalElementId, IntoElement, KeyDownEvent,
    LayoutId, PaintQuad, Pixels, Point, Render, ShapedLine, SharedString, Style, TextRun,
    UTF16Selection, Window, div, fill, point, prelude::*, px, relative, rgba, size,
};

pub struct TextDraftInput {
    focus_handle: FocusHandle,
    content: SharedString,
    selected_range: Range<usize>,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
}

impl TextDraftInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: "".into(),
            selected_range: 0..0,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn reset(&mut self, cx: &mut Context<Self>) {
        self.content = "".into();
        self.selected_range = 0..0;
        self.marked_range = None;
        self.last_layout = None;
        self.last_bounds = None;
        cx.notify();
    }

    fn offset_from_utf16_in(text: &str, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in text.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        Self::offset_from_utf16_in(&self.content, offset)
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end)
    }

    fn range_from_utf16_in(text: &str, range: &Range<usize>) -> Range<usize> {
        Self::offset_from_utf16_in(text, range.start)..Self::offset_from_utf16_in(text, range.end)
    }

    fn previous_char_boundary(&self, offset: usize) -> usize {
        self.content[..offset]
            .char_indices()
            .next_back()
            .map(|(index, _)| index)
            .unwrap_or(0)
    }

    fn next_char_boundary(&self, offset: usize) -> usize {
        self.content[offset..]
            .char_indices()
            .nth(1)
            .map(|(index, _)| offset + index)
            .unwrap_or(self.content.len())
    }

    fn delete_range(&mut self, range: Range<usize>, cx: &mut Context<Self>) {
        let content = self.content.to_string();
        self.content = (content[..range.start].to_owned() + &content[range.end..]).into();
        self.selected_range = range.start..range.start;
        self.marked_range = None;
        cx.notify();
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        if event.keystroke.is_ime_in_progress() {
            return;
        }

        match event.keystroke.key.as_str() {
            "backspace" => {
                if self.selected_range.start != self.selected_range.end {
                    self.delete_range(self.selected_range.clone(), cx);
                } else if self.selected_range.start > 0 {
                    let start = self.previous_char_boundary(self.selected_range.start);
                    self.delete_range(start..self.selected_range.start, cx);
                }
            }
            "delete" => {
                if self.selected_range.start != self.selected_range.end {
                    self.delete_range(self.selected_range.clone(), cx);
                } else if self.selected_range.end < self.content.len() {
                    let end = self.next_char_boundary(self.selected_range.end);
                    self.delete_range(self.selected_range.end..end, cx);
                }
            }
            "left" => {
                let cursor = if self.selected_range.start != self.selected_range.end {
                    self.selected_range.start
                } else {
                    self.previous_char_boundary(self.selected_range.start)
                };
                self.selected_range = cursor..cursor;
                self.marked_range = None;
                cx.notify();
            }
            "right" => {
                let cursor = if self.selected_range.start != self.selected_range.end {
                    self.selected_range.end
                } else {
                    self.next_char_boundary(self.selected_range.end)
                };
                self.selected_range = cursor..cursor;
                self.marked_range = None;
                cx.notify();
            }
            _ => {}
        }
    }
}

impl Focusable for TextDraftInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EntityInputHandler for TextDraftInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: false,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());
        let content = self.content.to_string();
        self.content =
            (content[..range.start].to_owned() + new_text + &content[range.end..]).into();
        let cursor = range.start + new_text.len();
        self.selected_range = cursor..cursor;
        self.marked_range = None;
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());
        let content = self.content.to_string();
        self.content =
            (content[..range.start].to_owned() + new_text + &content[range.end..]).into();
        self.marked_range =
            (!new_text.is_empty()).then_some(range.start..range.start + new_text.len());
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| Self::range_from_utf16_in(new_text, range_utf16))
            .map(|selected| range.start + selected.start..range.start + selected.end)
            .unwrap_or_else(|| {
                let cursor = range.start + new_text.len();
                cursor..cursor
            });
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let line = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(bounds.left() + line.x_for_index(range.start), bounds.top()),
            point(bounds.left() + line.x_for_index(range.end), bounds.bottom()),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point_value: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.last_bounds?;
        let line = self.last_layout.as_ref()?;
        let utf8 = line.closest_index_for_x(point_value.x - bounds.left());
        Some(self.offset_to_utf16(utf8))
    }
}

struct TextDraftElement {
    input: Entity<TextDraftInput>,
}

struct PrepaintState {
    line: ShapedLine,
    cursor: PaintQuad,
}

impl IntoElement for TextDraftElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextDraftElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> PrepaintState {
        let input = self.input.read(cx);
        let display: SharedString = if input.content.is_empty() {
            "Type annotation…".into()
        } else {
            input.content.clone()
        };
        let style = window.text_style();
        let run = TextRun {
            len: display.len(),
            font: style.font(),
            color: style.color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display, font_size, &[run], None);
        let cursor_x = line.x_for_index(input.selected_range.end.min(line.text.len()));
        let cursor = fill(
            Bounds::new(
                point(bounds.left() + cursor_x, bounds.top()),
                size(px(2.0), bounds.size.height),
            ),
            rgba(0xffffffff),
        );
        PrepaintState { line, cursor }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut (),
        prepaint: &mut PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        prepaint
            .line
            .paint(
                bounds.origin,
                window.line_height(),
                gpui::TextAlign::Left,
                None,
                window,
                cx,
            )
            .ok();
        if focus_handle.is_focused(window) {
            window.paint_quad(prepaint.cursor.clone());
        }
        let line = prepaint.line.clone();
        self.input.update(cx, |input, _| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
        });
    }
}

impl Render for TextDraftInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(220.0))
            .h(px(32.0))
            .px_2()
            .flex()
            .items_center()
            .rounded(px(6.0))
            .bg(rgba(0x303036ff))
            .border_1()
            .border_color(rgba(0x55555cff))
            .text_color(rgba(0xffffffff))
            .text_size(px(14.0))
            .cursor(CursorStyle::IBeam)
            .track_focus(&self.focus_handle(cx))
            .on_key_down(cx.listener(|input, event, _window, cx| {
                input.handle_key_down(event, cx);
            }))
            .child(TextDraftElement { input: cx.entity() })
    }
}

#[cfg(test)]
mod tests {
    use super::TextDraftInput;

    #[test]
    fn converts_utf16_offsets_against_inserted_text() {
        let inserted = "ก😀x";
        assert_eq!(TextDraftInput::offset_from_utf16_in(inserted, 0), 0);
        assert_eq!(TextDraftInput::offset_from_utf16_in(inserted, 1), 3);
        assert_eq!(TextDraftInput::offset_from_utf16_in(inserted, 3), 7);
        assert_eq!(TextDraftInput::offset_from_utf16_in(inserted, 4), 8);
    }

    #[test]
    fn converts_relative_utf16_selection_to_inserted_utf8_range() {
        let inserted = "😀ก";
        assert_eq!(TextDraftInput::range_from_utf16_in(inserted, &(2..3)), 4..7);
    }
}
