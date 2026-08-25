use anyhow::Result;
use global_hotkey::{GlobalHotKeyManager, hotkey::HotKey};
use sidekick_core::HotkeyBinding;

pub struct CaptureHotkeys {
    _manager: Option<GlobalHotKeyManager>,
    window_hotkey_id: Option<u32>,
    area_hotkey_id: Option<u32>,
}

impl CaptureHotkeys {
    pub fn new() -> Self {
        let manager = match GlobalHotKeyManager::new() {
            Ok(manager) => manager,
            Err(error) => {
                eprintln!("Screen Sidekick optional hotkey manager unavailable: {error}");
                return Self {
                    _manager: None,
                    window_hotkey_id: None,
                    area_hotkey_id: None,
                };
            }
        };

        let window_hotkey_id = register_default_hotkey(
            &manager,
            HotkeyBinding::window_default(),
            "focused-window capture",
        );
        let area_hotkey_id =
            register_default_hotkey(&manager, HotkeyBinding::area_default(), "area capture");

        Self {
            _manager: Some(manager),
            window_hotkey_id,
            area_hotkey_id,
        }
    }

    pub fn window_hotkey_id(&self) -> Option<u32> {
        self.window_hotkey_id
    }

    pub fn area_hotkey_id(&self) -> Option<u32> {
        self.area_hotkey_id
    }
}

fn register_default_hotkey(
    manager: &GlobalHotKeyManager,
    binding: HotkeyBinding,
    label: &str,
) -> Option<u32> {
    let hotkey = match runtime_hotkey(binding) {
        Ok(hotkey) => hotkey,
        Err(error) => {
            eprintln!("Screen Sidekick {label} hotkey is invalid: {error:#}");
            return None;
        }
    };
    let hotkey_id = hotkey.id();

    match manager.register(hotkey) {
        Ok(()) => Some(hotkey_id),
        Err(error) => {
            eprintln!("Screen Sidekick {label} hotkey unavailable: {error}");
            None
        }
    }
}

fn runtime_hotkey(binding: HotkeyBinding) -> Result<HotKey> {
    use global_hotkey::hotkey::Code;
    use sidekick_core::HotkeyKey;

    binding
        .validate()
        .map_err(|error| anyhow::anyhow!("invalid hotkey binding: {error:?}"))?;

    let modifiers = global_modifiers(binding.modifiers);
    let code = match binding.key {
        HotkeyKey::Character('0') => Code::Digit0,
        HotkeyKey::Character('1') => Code::Digit1,
        HotkeyKey::Character('2') => Code::Digit2,
        HotkeyKey::Character('3') => Code::Digit3,
        HotkeyKey::Character('4') => Code::Digit4,
        HotkeyKey::Character('5') => Code::Digit5,
        HotkeyKey::Character('6') => Code::Digit6,
        HotkeyKey::Character('7') => Code::Digit7,
        HotkeyKey::Character('8') => Code::Digit8,
        HotkeyKey::Character('9') => Code::Digit9,
        _ => {
            return Err(anyhow::anyhow!(
                "unsupported default hotkey key: {:?}",
                binding.key
            ));
        }
    };

    Ok(HotKey::new(Some(modifiers), code))
}

fn global_modifiers(modifiers: sidekick_core::HotkeyModifiers) -> global_hotkey::hotkey::Modifiers {
    use global_hotkey::hotkey::Modifiers;

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

#[cfg(test)]
mod tests {
    use super::*;
    use global_hotkey::hotkey::{Code, Modifiers};

    #[test]
    fn defaults_map_to_option_two_and_three() {
        let window = runtime_hotkey(HotkeyBinding::window_default()).unwrap();
        let area = runtime_hotkey(HotkeyBinding::area_default()).unwrap();

        assert_eq!(
            window.id(),
            HotKey::new(Some(Modifiers::ALT), Code::Digit2).id()
        );
        assert_eq!(
            area.id(),
            HotKey::new(Some(Modifiers::ALT), Code::Digit3).id()
        );
    }
}
