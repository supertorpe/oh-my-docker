use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::event::AppEvent;
use crate::app::state::AppState;

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    let search_active = state.navigation.logs.as_ref().map(|l| l.search_active).unwrap_or(false);
    if search_active {
        match (key.code, key.modifiers) {
            (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                state.navigation.logs.as_ref().map(|l| {
                    let new_q = l.search.chars().take(l.search.chars().count().saturating_sub(1)).collect();
                    AppEvent::SearchLogs(new_q)
                })
            }
            (KeyCode::Esc, _) => Some(AppEvent::SearchLogs(String::new())),
            (KeyCode::Enter, _) => Some(AppEvent::SubmitLogSearch),
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                state.navigation.logs.as_ref().map(|l| {
                    let new_q = format!("{}{}", l.search, c);
                    AppEvent::SearchLogs(new_q)
                })
            }
            _ => None,
        }
    } else {
        let km = &state.keymap;
        let code = key.code;
        let mods = key.modifiers;

        if km.is_toggle_selection(code, mods) || code == KeyCode::Char('p') {
            return Some(AppEvent::TogglePause);
        }
        if code == KeyCode::Char('r') {
            return state.navigation.logs.as_ref().and_then(|l| {
                if l.paused { Some(AppEvent::TogglePause) } else { None }
            });
        }
        if km.is_search(code, mods) {
            return Some(AppEvent::ActivateLogSearch);
        }
        if km.is_jump_top(code, mods) {
            return Some(AppEvent::JumpTop);
        }
        if km.is_jump_bottom(code, mods) {
            return Some(AppEvent::JumpBottom);
        }
        if code == KeyCode::Char('s') {
            return state.navigation.logs.as_ref().map(|l| AppEvent::ExportLogs(l.container_id.clone()));
        }
        if km.is_toggle_timestamps(code, mods) {
            return Some(AppEvent::ToggleLogTimestamps);
        }
        if km.is_logs_export(code, mods) {
            return state.navigation.logs.as_ref().map(|l| AppEvent::ExportLogs(l.container_id.clone()));
        }
        if code == KeyCode::Up || km.is_navigate_up(code, mods) {
            return Some(AppEvent::ScrollLogs(1));
        }
        if code == KeyCode::Down || km.is_navigate_down(code, mods) {
            return Some(AppEvent::ScrollLogs(-1));
        }
        if code == KeyCode::PageUp {
            return Some(AppEvent::ScrollLogs(20));
        }
        if code == KeyCode::PageDown {
            return Some(AppEvent::ScrollLogs(-20));
        }
        None
    }
}
