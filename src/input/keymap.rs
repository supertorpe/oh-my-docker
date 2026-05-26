use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Clone, Debug)]
pub struct ParsedKey {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl ParsedKey {
    pub fn matches(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.code == code && self.modifiers == modifiers
    }
}

/// Parse a keybinding string like "j", "Ctrl+A", "Esc", "Enter" into a ParsedKey.
pub fn parse_keybinding(s: &str) -> ParsedKey {
    let s = s.trim();
    match s.to_lowercase().as_str() {
        "esc" | "escape" => ParsedKey {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
        },
        "enter" => ParsedKey {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
        },
        "tab" => ParsedKey {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
        },
        "backspace" => ParsedKey {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
        },
        "up" | "arrowup" | "arrow_up" => ParsedKey {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
        },
        "down" | "arrowdown" | "arrow_down" => ParsedKey {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
        },
        "left" | "arrowleft" | "arrow_left" => ParsedKey {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
        },
        "right" | "arrowright" | "arrow_right" => ParsedKey {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
        },
        "pageup" | "page_up" => ParsedKey {
            code: KeyCode::PageUp,
            modifiers: KeyModifiers::NONE,
        },
        "pagedown" | "page_down" => ParsedKey {
            code: KeyCode::PageDown,
            modifiers: KeyModifiers::NONE,
        },
        "ctrl+a" | "ctrl_a" => ParsedKey {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+b" | "ctrl_b" => ParsedKey {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+c" | "ctrl_c" => ParsedKey {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+d" | "ctrl_d" => ParsedKey {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+e" | "ctrl_e" => ParsedKey {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+f" | "ctrl_f" => ParsedKey {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+h" | "ctrl_h" => ParsedKey {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+s" | "ctrl_s" => ParsedKey {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+u" | "ctrl_u" => ParsedKey {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+y" | "ctrl_y" => ParsedKey {
            code: KeyCode::Char('y'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+o" | "ctrl_o" => ParsedKey {
            code: KeyCode::Char('o'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+p" | "ctrl_p" => ParsedKey {
            code: KeyCode::Char('p'),
            modifiers: KeyModifiers::CONTROL,
        },
        "ctrl+t" | "ctrl_t" => ParsedKey {
            code: KeyCode::Char('t'),
            modifiers: KeyModifiers::CONTROL,
        },
        _ => {
            // Try to parse as a single character
            let chars: Vec<char> = s.chars().collect();
            if chars.len() == 1 {
                ParsedKey {
                    code: KeyCode::Char(chars[0]),
                    modifiers: KeyModifiers::NONE,
                }
            } else {
                // Default to the first character if it's not a special key
                ParsedKey {
                    code: KeyCode::Char(s.chars().next().unwrap_or('j')),
                    modifiers: KeyModifiers::NONE,
                }
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct KeyMap {
    pub quit: ParsedKey,
    pub back: ParsedKey,
    pub help: ParsedKey,
    pub navigate_down: ParsedKey,
    pub navigate_up: ParsedKey,
    pub open_details: ParsedKey,
    pub open_logs: ParsedKey,
    pub open_shell: ParsedKey,
    pub start_stop: ParsedKey,
    pub restart: ParsedKey,
    pub delete: ParsedKey,
    pub search: ParsedKey,
    pub toggle_selection: ParsedKey,
    pub select_all: ParsedKey,
    pub navigate_images: ParsedKey,
    pub run_image: ParsedKey,
    pub remove_image: ParsedKey,
    pub remove_dangling_images: ParsedKey,
    pub prune_images: ParsedKey,
    pub events_export: ParsedKey,
    pub jump_top: ParsedKey,
    pub jump_bottom: ParsedKey,
    pub statistics_sort: ParsedKey,
    pub statistics_sort_desc: ParsedKey,
    pub logs_export: ParsedKey,
    pub toggle_timestamps: ParsedKey,
    pub toggle_group_by_project: ParsedKey,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            quit: parse_keybinding("q"),
            back: parse_keybinding("Esc"),
            help: parse_keybinding("?"),
            navigate_down: parse_keybinding("j"),
            navigate_up: parse_keybinding("k"),
            open_details: parse_keybinding("Enter"),
            open_logs: parse_keybinding("l"),
            open_shell: parse_keybinding("s"),
            start_stop: parse_keybinding("t"),
            restart: parse_keybinding("r"),
            delete: parse_keybinding("d"),
            search: parse_keybinding("/"),
            toggle_selection: parse_keybinding(" "),
            select_all: parse_keybinding("Ctrl+A"),
            navigate_images: parse_keybinding("j"),
            run_image: parse_keybinding("r"),
            remove_image: parse_keybinding("d"),
            remove_dangling_images: parse_keybinding("D"),
            prune_images: parse_keybinding("p"),
            events_export: parse_keybinding("e"),
            jump_top: parse_keybinding("g"),
            jump_bottom: parse_keybinding("G"),
            statistics_sort: parse_keybinding("s"),
            statistics_sort_desc: parse_keybinding("S"),
            logs_export: parse_keybinding("Ctrl+S"),
            toggle_timestamps: parse_keybinding("T"),
            toggle_group_by_project: parse_keybinding("p"),
        }
    }
}

#[allow(dead_code)]
impl KeyMap {
    pub fn from_keybindings(keybindings: &crate::config::Keybindings) -> Self {
        Self {
            quit: parse_keybinding(&keybindings.quit),
            back: parse_keybinding(&keybindings.back),
            help: parse_keybinding(&keybindings.help),
            navigate_down: parse_keybinding(&keybindings.navigate_down),
            navigate_up: parse_keybinding(&keybindings.navigate_up),
            open_details: parse_keybinding(&keybindings.open_details),
            open_logs: parse_keybinding(&keybindings.open_logs),
            open_shell: parse_keybinding(&keybindings.open_shell),
            start_stop: parse_keybinding(&keybindings.start_stop),
            restart: parse_keybinding(&keybindings.restart),
            delete: parse_keybinding(&keybindings.delete),
            search: parse_keybinding(&keybindings.search),
            toggle_selection: parse_keybinding(&keybindings.toggle_selection),
            select_all: parse_keybinding(&keybindings.select_all),
            navigate_images: parse_keybinding(&keybindings.navigate_images),
            run_image: parse_keybinding(&keybindings.run_image),
            remove_image: parse_keybinding(&keybindings.remove_image),
            remove_dangling_images: parse_keybinding(&keybindings.remove_dangling_images),
            prune_images: parse_keybinding(&keybindings.prune_images),
            events_export: parse_keybinding(&keybindings.events_export),
            jump_top: parse_keybinding(&keybindings.jump_top),
            jump_bottom: parse_keybinding(&keybindings.jump_bottom),
            statistics_sort: parse_keybinding(&keybindings.statistics_sort),
            statistics_sort_desc: parse_keybinding(&keybindings.statistics_sort_desc),
            logs_export: parse_keybinding(&keybindings.logs_export),
            toggle_timestamps: parse_keybinding(&keybindings.toggle_timestamps),
            toggle_group_by_project: parse_keybinding(&keybindings.toggle_group_by_project),
        }
    }

    pub fn matches(&self, key: &crossterm::event::KeyEvent) -> bool {
        self.matches_code(key.code, key.modifiers)
    }

    pub fn matches_code(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        // Check if it's a global quit key
        if self.quit.matches(code, modifiers) {
            return true;
        }
        // Check if it's a global back key
        if self.back.matches(code, modifiers) {
            return true;
        }
        // Check if it's a global help key
        if self.help.matches(code, modifiers) {
            return true;
        }
        false
    }

    pub fn is_navigate_down(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.navigate_down.matches(code, modifiers)
    }

    pub fn is_navigate_up(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.navigate_up.matches(code, modifiers)
    }

    pub fn is_open_details(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.open_details.matches(code, modifiers)
    }

    pub fn is_open_logs(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.open_logs.matches(code, modifiers)
    }

    pub fn is_open_shell(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.open_shell.matches(code, modifiers)
    }

    pub fn is_start_stop(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.start_stop.matches(code, modifiers)
    }

    pub fn is_restart(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.restart.matches(code, modifiers)
    }

    pub fn is_delete(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.delete.matches(code, modifiers)
    }

    pub fn is_search(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.search.matches(code, modifiers)
    }

    pub fn is_toggle_selection(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.toggle_selection.matches(code, modifiers)
    }

    pub fn is_select_all(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.select_all.matches(code, modifiers)
    }

    pub fn is_navigate_images(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.navigate_images.matches(code, modifiers)
    }

    pub fn is_run_image(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.run_image.matches(code, modifiers)
    }

    pub fn is_remove_image(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.remove_image.matches(code, modifiers)
    }

    pub fn is_remove_dangling_images(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.remove_dangling_images.matches(code, modifiers)
    }

    pub fn is_prune_images(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.prune_images.matches(code, modifiers)
    }

    pub fn is_jump_top(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.jump_top.matches(code, modifiers)
    }

    pub fn is_jump_bottom(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.jump_bottom.matches(code, modifiers)
    }

    pub fn is_statistics_sort(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.statistics_sort.matches(code, modifiers)
    }

    pub fn is_statistics_sort_desc(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.statistics_sort_desc.matches(code, modifiers)
    }

    pub fn is_logs_export(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.logs_export.matches(code, modifiers)
    }

    pub fn is_toggle_timestamps(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.toggle_timestamps.matches(code, modifiers)
    }

    pub fn is_toggle_group_by_project(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.toggle_group_by_project.matches(code, modifiers)
    }

    pub fn is_help(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.help.matches(code, modifiers)
    }
}
