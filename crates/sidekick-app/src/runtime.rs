use anyhow::{Context as _, Result};
use global_hotkey::{
    GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem, PredefinedMenuItem},
};

pub struct AppRuntime {
    _tray: TrayIcon,
    _hotkey_manager: GlobalHotKeyManager,
}

impl AppRuntime {
    pub fn new() -> Result<Self> {
        let hotkey_manager = GlobalHotKeyManager::new().context("create global hotkey manager")?;
        let fullscreen_hotkey = HotKey::new(Some(Modifiers::ALT), Code::Digit1);
        hotkey_manager
            .register(fullscreen_hotkey)
            .context("register Option+1 fullscreen hotkey")?;

        let capture_item = MenuItem::new("Capture Fullscreen    ⌥1", true, None);
        let separator = PredefinedMenuItem::separator();
        let quit_item = MenuItem::new("Quit Screen Sidekick", true, None);

        let menu = Menu::new();
        menu.append_items(&[&capture_item, &separator, &quit_item])
            .context("build tray menu")?;

        let tray = TrayIconBuilder::new()
            .with_tooltip("Screen Sidekick")
            .with_icon(sidekick_tray_icon()?)
            .with_menu(Box::new(menu))
            .build()
            .context("create tray icon")?;

        Ok(Self {
            _tray: tray,
            _hotkey_manager: hotkey_manager,
        })
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

    // Small diagonal cut gives the otherwise simple screen glyph a distinct sidekick mark.
    for offset in 0..6 {
        let x = 7 + offset;
        let y = 6 + offset;
        let index = ((y * SIZE + x) * 4) as usize;
        rgba[index..index + 4].copy_from_slice(&[255, 255, 255, 255]);
    }

    Icon::from_rgba(rgba, SIZE, SIZE).context("create tray icon pixels")
}
