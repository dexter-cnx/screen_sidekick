#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotkeyAction {
    CaptureFullscreen,
    CaptureWindow,
    CaptureArea,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct HotkeyModifiers {
    pub control: bool,
    pub option: bool,
    pub shift: bool,
    pub command: bool,
}

impl HotkeyModifiers {
    pub const fn option() -> Self {
        Self {
            option: true,
            control: false,
            shift: false,
            command: false,
        }
    }

    pub const fn has_any(self) -> bool {
        self.control || self.option || self.shift || self.command
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotkeyKey {
    Character(char),
    Function(u8),
    Space,
    Enter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HotkeyBinding {
    pub action: HotkeyAction,
    pub modifiers: HotkeyModifiers,
    pub key: HotkeyKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyValidationError {
    MissingModifier,
    UnsupportedCharacter,
    FunctionKeyOutOfRange,
}

impl HotkeyBinding {
    pub const fn fullscreen_default() -> Self {
        Self {
            action: HotkeyAction::CaptureFullscreen,
            modifiers: HotkeyModifiers::option(),
            key: HotkeyKey::Character('1'),
        }
    }

    pub fn validate(self) -> Result<(), HotkeyValidationError> {
        if !self.modifiers.has_any() {
            return Err(HotkeyValidationError::MissingModifier);
        }

        match self.key {
            HotkeyKey::Character(character) if !character.is_ascii_alphanumeric() => {
                Err(HotkeyValidationError::UnsupportedCharacter)
            }
            HotkeyKey::Function(number) if !(1..=24).contains(&number) => {
                Err(HotkeyValidationError::FunctionKeyOutOfRange)
            }
            _ => Ok(()),
        }
    }

    pub const fn conflicts_with(self, other: Self) -> bool {
        self.modifiers.control == other.modifiers.control
            && self.modifiers.option == other.modifiers.option
            && self.modifiers.shift == other.modifiers.shift
            && self.modifiers.command == other.modifiers.command
            && hotkey_keys_equal(self.key, other.key)
    }
}

const fn hotkey_keys_equal(left: HotkeyKey, right: HotkeyKey) -> bool {
    match (left, right) {
        (HotkeyKey::Character(left), HotkeyKey::Character(right)) => left == right,
        (HotkeyKey::Function(left), HotkeyKey::Function(right)) => left == right,
        (HotkeyKey::Space, HotkeyKey::Space) | (HotkeyKey::Enter, HotkeyKey::Enter) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fullscreen_default_matches_option_one() {
        let binding = HotkeyBinding::fullscreen_default();

        assert_eq!(binding.action, HotkeyAction::CaptureFullscreen);
        assert_eq!(binding.modifiers, HotkeyModifiers::option());
        assert_eq!(binding.key, HotkeyKey::Character('1'));
        assert_eq!(binding.validate(), Ok(()));
    }

    #[test]
    fn binding_requires_at_least_one_modifier() {
        let binding = HotkeyBinding {
            action: HotkeyAction::CaptureArea,
            modifiers: HotkeyModifiers::default(),
            key: HotkeyKey::Character('A'),
        };

        assert_eq!(
            binding.validate(),
            Err(HotkeyValidationError::MissingModifier)
        );
    }

    #[test]
    fn character_keys_are_limited_to_ascii_alphanumeric() {
        let binding = HotkeyBinding {
            action: HotkeyAction::CaptureWindow,
            modifiers: HotkeyModifiers::option(),
            key: HotkeyKey::Character('/'),
        };

        assert_eq!(
            binding.validate(),
            Err(HotkeyValidationError::UnsupportedCharacter)
        );
    }

    #[test]
    fn function_keys_are_limited_to_f1_through_f24() {
        let binding = HotkeyBinding {
            action: HotkeyAction::CaptureWindow,
            modifiers: HotkeyModifiers::option(),
            key: HotkeyKey::Function(25),
        };

        assert_eq!(
            binding.validate(),
            Err(HotkeyValidationError::FunctionKeyOutOfRange)
        );
    }

    #[test]
    fn conflict_detection_ignores_action_and_compares_chord() {
        let fullscreen = HotkeyBinding::fullscreen_default();
        let area = HotkeyBinding {
            action: HotkeyAction::CaptureArea,
            ..fullscreen
        };

        assert!(fullscreen.conflicts_with(area));
    }
}
