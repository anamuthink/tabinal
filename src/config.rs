use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::Deserialize;

pub const DEFAULT_CONFIG_TEMPLATE: &str = "\
# tabinal configuration file
# All settings are optional. Uncomment values to customize.

[keybindings]
# quit              = \"ctrl+q\"
# new_tab           = [\"ctrl+t\", \"alt+t\"]
# rename_tab        = \"alt+n\"
# next_tab          = \"alt+right\"
# prev_tab          = \"alt+left\"
# focus_right_pane  = \"ctrl+right\"
# focus_left_pane   = \"ctrl+left\"
# focus_up_pane     = \"ctrl+up\"
# focus_down_pane   = \"ctrl+down\"
# split_right       = \"ctrl+shift+right\"
# split_down        = \"ctrl+shift+down\"
# toggle_file_tree  = \"ctrl+f\"
# close             = \"ctrl+w\"
# open_settings     = \"alt+s\"

[ui]
# icons = \"nerd\"   # \"nerd\" | \"plain\"
";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    RenameTab,
    NewTab,
    NextTab,
    PrevTab,
    FocusNextPane,
    FocusPrevPane,
    FocusRightPane,
    FocusLeftPane,
    FocusUpPane,
    FocusDownPane,
    SplitRight,
    SplitDown,
    ToggleFileTree,
    Close,
    OpenSettings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyChord {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

pub fn parse_key_chord(s: &str) -> Result<KeyChord, String> {
    let tokens: Vec<&str> = s.split('+').collect();
    if tokens.is_empty() {
        return Err(format!("empty key chord: {s:?}"));
    }

    let (modifier_tokens, key_token) = tokens.split_at(tokens.len() - 1);

    let mut modifiers = KeyModifiers::NONE;
    for tok in modifier_tokens {
        match tok.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
            "alt"              => modifiers |= KeyModifiers::ALT,
            "shift"            => modifiers |= KeyModifiers::SHIFT,
            other => return Err(format!("unknown modifier {other:?} in key chord {s:?}")),
        }
    }

    let code = match key_token[0].to_ascii_lowercase().as_str() {
        "right"    => KeyCode::Right,
        "left"     => KeyCode::Left,
        "up"       => KeyCode::Up,
        "down"     => KeyCode::Down,
        "home"     => KeyCode::Home,
        "end"      => KeyCode::End,
        "pageup"   => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "enter"    => KeyCode::Enter,
        "esc"      => KeyCode::Esc,
        "tab"      => KeyCode::Tab,
        "backspace"=> KeyCode::Backspace,
        "delete"   => KeyCode::Delete,
        s if s.chars().count() == 1 => {
            let c = s.chars().next().unwrap().to_ascii_lowercase();
            // For Char keys, SHIFT is implicit in the char value — strip it
            // so that "alt+shift+n" and "alt+n" resolve to the same chord.
            modifiers.remove(KeyModifiers::SHIFT);
            KeyCode::Char(c)
        }
        other => return Err(format!("unknown key {other:?} in key chord {s:?}")),
    };

    Ok(KeyChord { code, modifiers })
}

pub fn key_matches(chord: &KeyChord, key: &KeyEvent) -> bool {
    match chord.code {
        KeyCode::Char(c) => {
            let kc = match key.code {
                KeyCode::Char(x) => x.to_ascii_lowercase(),
                _ => return false,
            };
            let mods = key.modifiers - KeyModifiers::SHIFT;
            kc == c && mods == chord.modifiers
        }
        _ => key.code == chord.code && key.modifiers == chord.modifiers,
    }
}

// ─── TOML deserialization ────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    fn into_vec(self) -> Vec<String> {
        match self {
            OneOrMany::One(s) => vec![s],
            OneOrMany::Many(v) => v,
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct KeyBindingsConfig {
    quit:              Option<OneOrMany>,
    rename_tab:        Option<OneOrMany>,
    new_tab:           Option<OneOrMany>,
    next_tab:          Option<OneOrMany>,
    prev_tab:          Option<OneOrMany>,
    focus_next_pane:   Option<OneOrMany>,
    focus_prev_pane:   Option<OneOrMany>,
    focus_right_pane:  Option<OneOrMany>,
    focus_left_pane:   Option<OneOrMany>,
    focus_up_pane:     Option<OneOrMany>,
    focus_down_pane:   Option<OneOrMany>,
    split_right:       Option<OneOrMany>,
    split_down:        Option<OneOrMany>,
    toggle_file_tree:  Option<OneOrMany>,
    close:             Option<OneOrMany>,
    open_settings:     Option<OneOrMany>,
}

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IconStyle {
    #[default]
    Nerd,
    Plain,
}

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct UiSettings {
    pub icons: IconStyle,
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct Config {
    keybindings: KeyBindingsConfig,
    ui: UiSettings,
}

impl Config {
    fn resolve(self) -> Result<Vec<(KeyChord, Action)>, String> {
        let mut bindings = Self::defaults()?;

        let kb = self.keybindings;
        let overrides: Vec<(Option<OneOrMany>, Action)> = vec![
            (kb.quit,              Action::Quit),
            (kb.rename_tab,        Action::RenameTab),
            (kb.new_tab,           Action::NewTab),
            (kb.next_tab,          Action::NextTab),
            (kb.prev_tab,          Action::PrevTab),
            (kb.focus_next_pane,   Action::FocusNextPane),
            (kb.focus_prev_pane,   Action::FocusPrevPane),
            (kb.focus_right_pane,  Action::FocusRightPane),
            (kb.focus_left_pane,   Action::FocusLeftPane),
            (kb.focus_up_pane,     Action::FocusUpPane),
            (kb.focus_down_pane,   Action::FocusDownPane),
            (kb.split_right,       Action::SplitRight),
            (kb.split_down,        Action::SplitDown),
            (kb.toggle_file_tree,  Action::ToggleFileTree),
            (kb.close,             Action::Close),
            (kb.open_settings,     Action::OpenSettings),
        ];

        for (entry, action) in overrides {
            let Some(one_or_many) = entry else { continue };
            // Replace all defaults for this action
            bindings.retain(|(_, a)| *a != action);
            for key_str in one_or_many.into_vec() {
                let chord = parse_key_chord(&key_str)
                    .map_err(|e| format!("invalid keybinding: {e}"))?;
                bindings.push((chord, action));
            }
        }

        Ok(bindings)
    }

    fn defaults() -> Result<Vec<(KeyChord, Action)>, String> {
        let pairs: &[(&str, Action)] = &[
            ("ctrl+q",           Action::Quit),
            ("alt+n",            Action::RenameTab),
            ("ctrl+t",           Action::NewTab),
            ("alt+t",            Action::NewTab),
            ("alt+right",        Action::NextTab),
            ("alt+left",         Action::PrevTab),
            ("ctrl+right",       Action::FocusRightPane),
            ("ctrl+left",        Action::FocusLeftPane),
            ("ctrl+up",          Action::FocusUpPane),
            ("ctrl+down",        Action::FocusDownPane),
            ("ctrl+shift+right", Action::SplitRight),
            ("ctrl+shift+down",  Action::SplitDown),
            ("ctrl+f",           Action::ToggleFileTree),
            ("ctrl+w",           Action::Close),
            ("alt+s",            Action::OpenSettings),
        ];

        pairs.iter()
            .map(|(s, a)| parse_key_chord(s).map(|c| (c, *a)))
            .collect()
    }
}

pub struct LoadedConfig {
    pub keybindings: Vec<(KeyChord, Action)>,
    pub ui: UiSettings,
}

pub fn config_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("tabinal").join("config.toml"))
}

pub fn load_config() -> Result<LoadedConfig, Box<dyn std::error::Error>> {
    let config = load_config_file()?;
    let ui = config.ui.clone();
    let keybindings = config.resolve().map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    Ok(LoadedConfig { keybindings, ui })
}

fn load_config_file() -> Result<Config, Box<dyn std::error::Error>> {
    let Some(dir) = dirs::config_dir() else {
        return Ok(Config::default());
    };
    let path = dir.join("tabinal").join("config.toml");
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = std::fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&content)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(config)
}

