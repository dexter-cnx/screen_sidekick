use anyhow::{Context as _, Result};
use global_hotkey::{GlobalHotKeyManager, hotkey::HotKey};
use sidekick_core::HotkeyBinding;

pub struct CaptureHotkeys {
    _manager: GlobalHotKeyManager,
    window_hotkey_id: u32,
    area_hotkey_id: u32,
}

impl CaptureHotkeys {
    pub fn new() -> Result<Self> {
        let manager = GlobalHotKeyManager::new().context("create window/area hotkey manager")?;
        let window_hotkey = runtime_hotkey(HotkeyBinding::window_default())?;
        let area_hotkey = runtime_hotkey(HotkeyBinding::area_default())?;
        let window_hotkey_id = window_hotkey.id();
        let area_hotkey_id = area_hotkey.id();

        manager
            .register(window_hotkey)
            .context("register focused-window capture hotkey")?;
        if let Err(error) = manager.register(area_hotkey) {
            let _ = manager.unregister(window_hotkey);
            return Err(error).context("register area capture hotkey");
        }

        Ok(Self {
            _manager: manager,
            window_hotkey_id,
            area_hotkey_id,
        })
    }

    pub fn window_hotkey_id(&self) -> u32 {
        self.window_hotkey_id
    }

    pub fn area_hotkey_id(&self) -> u32 {
        self.area_hotkey_id
    }
}

fn runtime_hotkey(binding: HotkeyBinding) -> Result<HotKey> {
    use global_hotkey::hotkey::{Code, Modifiers};
    use sidekick_core::{HotkeyKey, HotkeyModifiers};

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
        _ => return Err(anyhow::anyhow!("unsupported default hotkey key: {:?}", binding.key)),
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

        assert_eq!(window.id(), HotKey::new(Some(Modifiers::ALT), Code::Digit2).id());
        assert_eq!(area.id(), HotKey::new(Some(Modifiers::ALT), Code::Digit3).id());
    }
}
