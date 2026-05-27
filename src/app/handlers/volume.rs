use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::event::{AppEvent, ConfirmAction};
use crate::app::state::AppState;

pub fn handle_key_with_clipboard(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('y') {
        if let Some(v) = state.volumes.items.get(state.volumes.selected) {
            if crate::util::copy_to_clipboard(&v.name) {
                return Some(AppEvent::Info(format!("Volume name copied to clipboard")));
            } else {
                return Some(AppEvent::Info(format!("Failed to copy to clipboard - install xclip, wl-copy, or xsel")));
            }
        }
    }
    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('o') {
        return Some(AppEvent::ToggleColumnPicker);
    }
    handle_key(key, state)
}

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if state.volumes.show_column_picker {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => Some(AppEvent::ToggleColumnPicker),
            (KeyCode::Char(' '), _) | (KeyCode::Enter, _) => {
                let names = ["name", "driver", "mountpoint"];
                let idx = state.volumes.column_picker_selection.min(names.len() - 1);
                Some(AppEvent::ToggleColumn(names[idx].to_string()))
            }
            _ if state.keymap.is_navigate_down(key.code, key.modifiers) || key.code == KeyCode::Down => {
                Some(AppEvent::ToggleColumn("next".to_string()))
            }
            _ if state.keymap.is_navigate_up(key.code, key.modifiers) || key.code == KeyCode::Up => {
                Some(AppEvent::ToggleColumn("prev".to_string()))
            }
            _ => None,
        }
    } else {
        let km = &state.keymap;
        let code = key.code;
        let mods = key.modifiers;

        if code == KeyCode::Esc {
            return Some(AppEvent::Back);
        }
        if km.is_navigate_down(code, mods) || code == KeyCode::Down {
            let next = (state.volumes.selected + 1).min(state.volumes.items.len().saturating_sub(1));
            return Some(AppEvent::SelectVolume(next));
        }
        if km.is_navigate_up(code, mods) || code == KeyCode::Up {
            let prev = state.volumes.selected.saturating_sub(1);
            return Some(AppEvent::SelectVolume(prev));
        }
        if km.is_delete(code, mods) {
            return state.volumes.items.get(state.volumes.selected)
                .map(|v| AppEvent::ShowConfirmDialog(
                    format!("Remove volume '{}'?", v.name),
                    ConfirmAction::RemoveVolume(v.name.clone()),
                ));
        }
        None
    }
}
