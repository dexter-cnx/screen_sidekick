use anyhow::{Context as _, Result, anyhow};
use global_hotkey::{
    GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
use sidekick_core::{HotkeyBinding, HotkeyKey, HotkeyModifiers};
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuId, MenuItem, PredefinedMenuItem},
};

pub struct AppRuntime {
    _tray: TrayIcon,
    hotkey_manager: GlobalHotKeyManager,
    capture_menu_id: MenuId,
    settings_menu_id: MenuId,
    quit_menu_id: MenuId,
    fullscreen_binding: HotkeyBinding,
    fullscreen_hotkey: HotKey,
    fullscreen_hotkey_id: u32,
}

impl AppRuntime {
    pub fn new() -> Result<Self> {
        let hotkey_manager = GlobalHotKeyManager::new().context("create global hotkey manager")?;
        let fullscreen_binding = HotkeyBinding::fullscreen_default();
        let fullscreen_hotkey = hotkey_from_binding(fullscreen_binding)?;
        let fullscreen_hotkey_id = fullscreen_hotkey.id();
        hotkey_manager
            .register(fullscreen_hotkey.clone())
            .context("register fullscreen capture hotkey")?;

        let capture_item = MenuItem::new("Capture Fullscreen    ⌥1", true, None);
        let settings_item = MenuItem::new("Settings…", true, None);
        let separator = PredefinedMenuItem::separator();
        let quit_item = MenuItem::new("Quit Screen Sidekick", true, None);
        let capture_menu_id = capture_item.id().clone();
        let settings_menu_id = settings_item.id().clone();
        let quit_menu_id = quit_item.id().clone();

        let menu = Menu::new();
        menu.append_items(&[&capture_item, &settings_item, &separator, &quit_item])
            .context("build tray menu")?;

        let tray = TrayIconBuilder::new()
            .with_tooltip("Screen Sidekick")
            .with_icon(sidekick_tray_icon()?)
            .with_menu(Box::new(menu))
            .build()
            .context("create tray icon")?;

        Ok(Self {
            _tray: tray,
            hotkey_manager,
            capture_menu_id,
            settings_menu_id,
            quit_menu_id,
            fullscreen_binding,
            fullscreen_hotkey,
            fullscreen_hotkey_id,
        })
    }

    pub fn capture_menu_id(&self) -> &MenuId {
        &self.capture_menu_id
    }

    pub fn settings_menu_id(&self) -> &MenuId {
        &self.settings_menu_id
    }

    pub fn quit_menu_id(&self) -> &MenuId {
        &self.quit_menu_id
    }

    pub fn fullscreen_binding(&self) -> HotkeyBinding {
        self.fullscreen_binding
    }

    pub fn fullscreen_hotkey_id(&self) -> u32 {
        self.fullscreen_hotkey_id
    }

    pub fn set_fullscreen_binding(&mut self, binding: HotkeyBinding) -> Result<()> {
        let next_hotkey = hotkey_from_binding(binding)?;
        if next_hotkey.id() == self.fullscreen_hotkey_id {
            self.fullscreen_binding = binding;
            return Ok(());
        }

        let previous_hotkey = self.fullscreen_hotkey.clone();
        self.hotkey_manager
            .unregister(previous_hotkey.clone())
            .context("unregister previous fullscreen hotkey")?;

        if let Err(error) = self.hotkey_manager.register(next_hotkey.clone()) {
            let _ = self.hotkey_manager.register(previous_hotkey);
            return Err(error).context("register replacement fullscreen hotkey");
        }

        self.fullscreen_binding = binding;
        self.fullscreen_hotkey_id = next_hotkey.id();
        self.fullscreen_hotkey = next_hotkey;
        Ok(())
    }
}

fn hotkey_from_binding(binding: HotkeyBinding) -> Result<HotKey> {
    binding
        .validate()
        .map_err(|error| anyhow!("invalid hotkey binding: {error:?}"))?;

    let modifiers = global_modifiers(binding.modifiers);
    let code = global_code(binding.key)
        .ok_or_else(|| anyhow!("unsupported hotkey key: {:?}", binding.key))?;

    Ok(HotKey::new(Some(modifiers), code))
}

fn global_modifiers(modifiers: HotkeyModifiers) -> Modifiers {
    let mut result = Modifiers::empty();
    if modifiers.control {
        result |= Modifiers::CONTROL;
    }
    if modifiers.option {
        result |= Modifiers::ALT;
    }
    if modifiers.shift {
        result |= Modifiers::SHIFT;
    }
    if modifiers.command {
        result |= Modifiers::SUPER;
    }
    result
}

fn global_code(key: HotkeyKey) -> Option<Code> {
    match key {
        HotkeyKey::Character(character) => match character.to_ascii_uppercase() {
            '0' => Some(Code::Digit0),
            '1' => Some(Code::Digit1),
            '2' => Some(Code::Digit2),
            '3' => Some(Code::Digit3),
            '4' => Some(Code::Digit4),
            '5' => Some(Code::Digit5),
            '6' => Some(Code::Digit6),
            '7' => Some(Code::Digit7),
            '8' => Some(Code::Digit8),
            '9' => Some(Code::Digit9),
            'A' => Some(Code::KeyA),
            'B' => Some(Code::KeyB),
            'C' => Some(Code::KeyC),
            'D' => Some(Code::KeyD),
            'E' => Some(Code::KeyE),
            'F' => Some(Code::KeyF),
            'G' => Some(Code::KeyG),
            'H' => Some(Code::KeyH),
            'I' => Some(Code::KeyI),
            'J' => Some(Code::KeyJ),
            'K' => Some(Code::KeyK),
            'L' => Some(Code::KeyL),
            'M' => Some(Code::KeyM),
            'N' => Some(Code::KeyN),
            'O' => Some(Code::KeyO),
            'P' => Some(Code::KeyP),
            'Q' => Some(Code::KeyQ),
            'R' => Some(Code::KeyR),
            'S' => Some(Code::KeyS),
            'T' => Some(Code::KeyT),
            'U' => Some(Code::KeyU),
            'V' => Some(Code::KeyV),
            'W' => Some(Code::KeyW),
            'X' => Some(Code::KeyX),
            'Y' => Some(Code::KeyY),
            'Z' => Some(Code::KeyZ),
            _ => None,
        },
        HotkeyKey::Function(number) => function_code(number),
        HotkeyKey::Space => Some(Code::Space),
        HotkeyKey::Enter => Some(Code::Enter),
    }
}

fn function_code(number: u8) -> Option<Code> {
    match number {
        1 => Some(Code::F1),
        2 => Some(Code::F2),
        3 => Some(Code::F3),
        4 => Some(Code::F4),
        5 => Some(Code::F5),
        6 => Some(Code::F6),
        7 => Some(Code::F7),
        8 => Some(Code::F8),
        9 => Some(Code::F9),
        10 => Some(Code::F10),
        11 => Some(Code::F11),
        12 => Some(Code::F12),
        13 => Some(Code::F13),
        14 => Some(Code::F14),
        15 => Some(Code::F15),
        16 => Some(Code::F16),
        17 => Some(Code::F17),
        18 => Some(Code::F18),
        19 => Some(Code::F19),
        20 => Some(Code::F20),
        21 => Some(Code::F21),
        22 => Some(Code::F22),
        23 => Some(Code::F23),
        24 => Some(Code::F24),
        _ => None,
    }
}

fn sidekick_tray_icon() -> Result<Icon> {
    const SIZE: u32 = 18;
    let mut rgba = vec![0_u8; (SIZE * SIZE * 4) as usize];

    for y in 3..15 {
        for x in 2..16 {
            let border = x == 2 || x == 15 || y == 3 || y == 14;
            if border {
                let index = ((y * SIZE + x) * 4) as usize;
                rgba[index..index + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
    }

    for offset in 0..6 {
        let x = 7 + offset;
        let y = 6 + offset;
        let index = ((y * SIZE + x) * 4) as usize;
        rgba[index..index + 4].copy_from_slice(&[255, 255, 255, 255]);
    }

    Icon::from_rgba(rgba, SIZE, SIZE).context("create tray icon pixels")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sidekick_core::{HotkeyAction, HotkeyModifiers};

    #[test]
    fn fullscreen_default_maps_to_option_digit_one() {
        let mapped = hotkey_from_binding(HotkeyBinding::fullscreen_default()).unwrap();
        let expected = HotKey::new(Some(Modifiers::ALT), Code::Digit1);

        assert_eq!(mapped.id(), expected.id());
    }

    #[test]
    fn character_mapping_is_case_insensitive() {
        let uppercase = HotkeyBinding {
            action: HotkeyAction::CaptureWindow,
            modifiers: HotkeyModifiers::option(),
            key: HotkeyKey::Character('A'),
        };
        let lowercase = HotkeyBinding {
            key: HotkeyKey::Character('a'),
            ..uppercase
        };

        assert_eq!(
            hotkey_from_binding(uppercase).unwrap().id(),
            hotkey_from_binding(lowercase).unwrap().id()
        );
    }

    #[test]
    fn all_supported_modifier_flags_map_to_global_hotkey() {
        let mapped = global_modifiers(HotkeyModifiers {
            control: true,
            option: true,
            shift: true,
            command: true,
        });

        assert!(mapped.contains(Modifiers::CONTROL));
        assert!(mapped.contains(Modifiers::ALT));
        assert!(mapped.contains(Modifiers::SHIFT));
        assert!(mapped.contains(Modifiers::SUPER));
    }
}
