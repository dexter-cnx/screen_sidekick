use gpui::{
    App, Bounds, Context, FocusHandle, Focusable, KeyDownEvent, Render, TitlebarOptions, Window,
    WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};
use sidekick_core::{HotkeyAction, HotkeyBinding, HotkeyKey, HotkeyModifiers};
use std::sync::mpsc::Sender;

const SETTINGS_WIDTH: f32 = 460.0;
const SETTINGS_HEIGHT: f32 = 260.0;

pub struct HotkeySettingsView {
    binding: HotkeyBinding,
    status: String,
    binding_sender: Sender<HotkeyBinding>,
    focus_handle: FocusHandle,
}

impl HotkeySettingsView {
    pub fn new(
        binding: HotkeyBinding,
        binding_sender: Sender<HotkeyBinding>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);
        Self {
            binding,
            status: "Press a new shortcut".to_owned(),
            binding_sender,
            focus_handle,
        }
    }

    pub fn apply_binding_result(
        &mut self,
        result: Result<HotkeyBinding, String>,
        cx: &mut Context<Self>,
    ) {
        match result {
            Ok(binding) => {
                self.binding = binding;
                self.status = "Shortcut updated".to_owned();
            }
            Err(error) => {
                self.status = format!("Shortcut unchanged: {error}");
            }
        }
        cx.notify();
    }

    fn record(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let Some(key) = hotkey_key_from_gpui(&event.keystroke.key) else {
            self.status = format!("Unsupported key: {}", event.keystroke.key);
            cx.notify();
            return;
        };

        let modifiers = HotkeyModifiers {
            control: event.keystroke.modifiers.control,
            option: event.keystroke.modifiers.alt,
            shift: event.keystroke.modifiers.shift,
            command: event.keystroke.modifiers.platform,
        };
        let binding = HotkeyBinding {
            action: HotkeyAction::CaptureFullscreen,
            modifiers,
            key,
        };

        if let Err(error) = binding.validate() {
            self.status = format!("Invalid shortcut: {error:?}");
            cx.notify();
            return;
        }

        match self.binding_sender.send(binding) {
            Ok(()) => self.status = "Applying shortcut…".to_owned(),
            Err(error) => self.status = format!("Shortcut unchanged: {error}"),
        }
        cx.notify();
    }
}

impl Focusable for HotkeySettingsView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for HotkeySettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let shortcut = format_binding(self.binding);
        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|view, event: &KeyDownEvent, _, cx| {
                view.record(event, cx);
            }))
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .bg(rgb(0x15131b))
            .text_color(rgb(0xe8e5ef))
            .child(div().text_xl().child("Hotkeys"))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xa9a3b4))
                    .child("Fullscreen capture"),
            )
            .child(
                div()
                    .px_4()
                    .py_3()
                    .rounded(px(10.0))
                    .border_1()
                    .border_color(rgb(0x4b4558))
                    .bg(rgb(0x211e29))
                    .text_lg()
                    .child(shortcut),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x8f8999))
                    .child("Press the replacement shortcut while this window is focused."),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xbcb6c7))
                    .child(self.status.clone()),
            )
    }
}

pub fn settings_window_options(cx: &App) -> WindowOptions {
    let window_size = size(px(SETTINGS_WIDTH), px(SETTINGS_HEIGHT));
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            window_size,
            cx,
        ))),
        titlebar: Some(TitlebarOptions {
            title: Some("Screen Sidekick Settings".into()),
            ..Default::default()
        }),
        focus: true,
        is_movable: true,
        is_resizable: false,
        ..Default::default()
    }
}

fn hotkey_key_from_gpui(key: &str) -> Option<HotkeyKey> {
    let normalized = key.to_ascii_lowercase();
    if normalized.len() == 1 {
        let character = normalized.chars().next()?;
        return character
            .is_ascii_alphanumeric()
            .then_some(HotkeyKey::Character(character));
    }
    match normalized.as_str() {
        "space" => Some(HotkeyKey::Space),
        "enter" => Some(HotkeyKey::Enter),
        _ => normalized
            .strip_prefix('f')
            .and_then(|value| value.parse::<u8>().ok())
            .filter(|number| (1..=24).contains(number))
            .map(HotkeyKey::Function),
    }
}

fn format_binding(binding: HotkeyBinding) -> String {
    let mut parts = Vec::new();
    if binding.modifiers.control {
        parts.push("⌃".to_owned());
    }
    if binding.modifiers.option {
        parts.push("⌥".to_owned());
    }
    if binding.modifiers.shift {
        parts.push("⇧".to_owned());
    }
    if binding.modifiers.command {
        parts.push("⌘".to_owned());
    }
    parts.push(match binding.key {
        HotkeyKey::Character(character) => character.to_ascii_uppercase().to_string(),
        HotkeyKey::Function(number) => format!("F{number}"),
        HotkeyKey::Space => "Space".to_owned(),
        HotkeyKey::Enter => "Enter".to_owned(),
    });
    parts.join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_gpui_keys_to_core_keys() {
        assert_eq!(hotkey_key_from_gpui("a"), Some(HotkeyKey::Character('a')));
        assert_eq!(hotkey_key_from_gpui("F12"), Some(HotkeyKey::Function(12)));
        assert_eq!(hotkey_key_from_gpui("space"), Some(HotkeyKey::Space));
        assert_eq!(hotkey_key_from_gpui("escape"), None);
    }

    #[test]
    fn formats_mac_shortcut_symbols() {
        assert_eq!(format_binding(HotkeyBinding::fullscreen_default()), "⌥1");
    }
}
