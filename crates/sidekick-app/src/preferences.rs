use sidekick_core::WindowShadowPolicy;
use std::{fs, io, path::PathBuf};

const SHADOW_POLICY_FILE: &str = "window-shadow-policy";

pub fn load_window_shadow_policy() -> WindowShadowPolicy {
    let Some(path) = preference_path() else {
        return WindowShadowPolicy::Include;
    };

    match fs::read_to_string(path) {
        Ok(value) if value.trim() == "exclude" => WindowShadowPolicy::Exclude,
        Ok(_) => WindowShadowPolicy::Include,
        Err(error) if error.kind() == io::ErrorKind::NotFound => WindowShadowPolicy::Include,
        Err(error) => {
            eprintln!("Screen Sidekick preference read failed: {error}");
            WindowShadowPolicy::Include
        }
    }
}

pub fn save_window_shadow_policy(policy: WindowShadowPolicy) {
    let Some(path) = preference_path() else {
        return;
    };

    if let Some(parent) = path.parent()
        && let Err(error) = fs::create_dir_all(parent)
    {
        eprintln!("Screen Sidekick preference directory failed: {error}");
        return;
    }

    let value = match policy {
        WindowShadowPolicy::Include => "include\n",
        WindowShadowPolicy::Exclude => "exclude\n",
    };
    if let Err(error) = fs::write(path, value) {
        eprintln!("Screen Sidekick preference write failed: {error}");
    }
}

fn preference_path() -> Option<PathBuf> {
    dirs::config_dir().map(|root| root.join("Screen Sidekick").join(SHADOW_POLICY_FILE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persisted_values_are_stable() {
        assert_eq!(
            match WindowShadowPolicy::Include {
                WindowShadowPolicy::Include => "include",
                WindowShadowPolicy::Exclude => "exclude",
            },
            "include"
        );
        assert_eq!(
            match WindowShadowPolicy::Exclude {
                WindowShadowPolicy::Include => "include",
                WindowShadowPolicy::Exclude => "exclude",
            },
            "exclude"
        );
    }
}
