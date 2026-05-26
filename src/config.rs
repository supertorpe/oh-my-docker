use std::collections::HashMap;
use std::path::PathBuf;
use crate::input::keymap::KeyMap;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ContainerShellConfig {
    pub shell: Option<String>,
    pub user: Option<String>,
    pub workdir: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Keybindings {
    pub quit: String,
    pub back: String,
    pub help: String,
    pub navigate_down: String,
    pub navigate_up: String,
    pub open_details: String,
    pub open_logs: String,
    pub open_shell: String,
    pub start_stop: String,
    pub restart: String,
    pub delete: String,
    pub search: String,
    pub toggle_selection: String,
    pub select_all: String,
    pub navigate_images: String,
    pub run_image: String,
    pub remove_image: String,
    pub remove_dangling_images: String,
    pub prune_images: String,
    pub events_export: String,
    pub jump_top: String,
    pub jump_bottom: String,
    pub statistics_sort: String,
    pub statistics_sort_desc: String,
    pub logs_export: String,
    pub toggle_timestamps: String,
    pub toggle_group_by_project: String,
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            quit: "q".to_string(),
            back: "Esc".to_string(),
            help: "?".to_string(),
            navigate_down: "j".to_string(),
            navigate_up: "k".to_string(),
            open_details: "Enter".to_string(),
            open_logs: "l".to_string(),
            open_shell: "s".to_string(),
            start_stop: "t".to_string(),
            restart: "r".to_string(),
            delete: "d".to_string(),
            search: "/".to_string(),
            toggle_selection: " ".to_string(),
            select_all: "Ctrl+A".to_string(),
            navigate_images: "j".to_string(),
            run_image: "r".to_string(),
            remove_image: "d".to_string(),
            remove_dangling_images: "D".to_string(),
            prune_images: "p".to_string(),
            events_export: "e".to_string(),
            jump_top: "g".to_string(),
            jump_bottom: "G".to_string(),
            statistics_sort: "s".to_string(),
            statistics_sort_desc: "S".to_string(),
            logs_export: "Ctrl+S".to_string(),
            toggle_timestamps: "T".to_string(),
            toggle_group_by_project: "p".to_string(),
        }
    }
}

impl Keybindings {
    pub fn to_help_text(&self) -> Vec<String> {
        vec![
            format!("    {}     Quit", self.quit),
            format!("    {}     Toggle help", self.help),
            format!("    {}     Go back", self.back),
            "".to_string(),
            "  CONTAINERS".to_string(),
            format!("    {} / {}   Navigate down", self.navigate_down, "↓"),
            format!("    {} / {}   Navigate up", self.navigate_up, "↑"),
            format!("    {}     Open details", self.open_details),
            format!("    {}     Open logs", self.open_logs),
            format!("    {}     Open shell", self.open_shell),
            format!("    {}     Start/Stop container", self.start_stop),
            format!("    {}     Restart container", self.restart),
            format!("    {}     Delete container", self.delete),
            format!("    {}     Search", self.search),
            format!("    {}     Images view", "i"),
            format!("    {}     Events view", "e"),
            format!("    {}     Statistics view", "%"),
            format!("    {}     Networks view", "n"),
            format!("    {}     Volumes view", "v"),
            "".to_string(),
            "  IMAGES".to_string(),
            format!("    {} / {}   Navigate down", self.navigate_images, "↓"),
            format!("    {} / {}   Navigate up", self.navigate_up, "↑"),
            format!("    {}     Run image", self.run_image),
            format!("    {}     Remove image", self.remove_image),
            format!("    {}     Remove dangling images", self.remove_dangling_images),
            format!("    {}     Prune unused images", self.prune_images),
            format!("    {}     Search", self.search),
            "".to_string(),
            "  LOGS".to_string(),
            format!("    {} / {}   Scroll down", self.navigate_down, "↓"),
            format!("    {} / {}   Scroll up", self.navigate_up, "↑"),
            "    PgDn  Page down".to_string(),
            "    PgUp  Page up".to_string(),
            format!("    {}     Jump to bottom", self.jump_bottom),
            format!("    {}     Jump to top", self.jump_top),
            format!("    {}     Search logs", self.search),
            "    Space Pause/unpause".to_string(),
            "".to_string(),
            "  EVENTS".to_string(),
            format!("    {}     Filter events", self.search),
            "".to_string(),
            "  STATISTICS".to_string(),
            format!("    {}     Cycle sort field", self.statistics_sort),
            format!("    {}     Toggle sort direction", self.statistics_sort_desc),
            "".to_string(),
            "  NETWORKS / VOLUMES".to_string(),
            format!("    {} / {}   Navigate down", self.navigate_down, "↓"),
            format!("    {} / {}   Navigate up", self.navigate_up, "↑"),
            format!("    {}     Delete selected", self.delete),
            "".to_string(),
            "  SCROLLING".to_string(),
            format!("    {} / {}   Scroll down", self.navigate_down, "↓"),
            format!("    {} / {}   Scroll up", self.navigate_up, "↑"),
            "    PgDn  Page down".to_string(),
            "    PgUp  Page up".to_string(),
            format!("    {}     Jump to bottom", self.jump_bottom),
            format!("    {}     Jump to top", self.jump_top),
            "".to_string(),
            format!("  Press {} or {} to close", self.back, self.help),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OmdockerConfig {
    pub latest_shell: Option<String>,
    #[serde(default)]
    pub containers: HashMap<String, ContainerShellConfig>,
    #[serde(default)]
    pub images: HashMap<String, ContainerShellConfig>,
    #[serde(default)]
    pub check_updates: Option<bool>,
    #[serde(default)]
    pub keybindings: Keybindings,
}

impl OmdockerConfig {
    pub fn keymap(&self) -> KeyMap {
        KeyMap::from_keybindings(&self.keybindings)
    }
}

impl Default for OmdockerConfig {
    fn default() -> Self {
        Self {
            latest_shell: Some("bash".to_string()),
            containers: HashMap::new(),
            images: HashMap::new(),
            check_updates: None,
            keybindings: Keybindings::default(),
        }
    }
}

impl OmdockerConfig {
    fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config").join("omdocker").join("omdocker.toml")
    }

    pub fn load() -> Self {
        let path = Self::path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = toml::to_string(self) {
            let _ = std::fs::write(&path, content);
        }
    }
}
