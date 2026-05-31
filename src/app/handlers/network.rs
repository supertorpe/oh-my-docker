use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::event::{AppEvent, ConfirmAction};
use crate::app::state::AppState;
use crate::ui::resource_panel::{NetworkResource, Resource};

pub fn handle_key_with_clipboard(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('y') {
        if let Some(n) = state.networks.items.get(state.networks.selected) {
            return crate::app::handlers::clipboard_copy(&n.id);
        }
    }
    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('o') {
        return Some(AppEvent::ToggleColumnPicker);
    }
    handle_key(key, state)
}

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if state.networks.show_column_picker {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => Some(AppEvent::ToggleColumnPicker),
            (KeyCode::Char(' '), _) | (KeyCode::Enter, _) => {
                let names = ["name", "id", "driver", "scope", "ipam"];
                let idx = state.networks.column_picker_selection.min(names.len() - 1);
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
            let next = (state.networks.selected + 1).min(state.networks.items.len().saturating_sub(1));
            return Some(AppEvent::SelectNetwork(next));
        }
        if km.is_navigate_up(code, mods) || code == KeyCode::Up {
            let prev = state.networks.selected.saturating_sub(1);
            return Some(AppEvent::SelectNetwork(prev));
        }
        if km.is_sort_direction(code, mods) {
            return Some(AppEvent::ToggleSortDirection);
        }
        if code == KeyCode::Left {
            let n = NetworkResource::column_headers().len();
            let next = ((state.networks.sort_column as i32 - 1).rem_euclid(n as i32)) as usize;
            return Some(AppEvent::SortByColumn(next));
        }
        if code == KeyCode::Right {
            let n = NetworkResource::column_headers().len();
            let next = ((state.networks.sort_column as i32 + 1).rem_euclid(n as i32)) as usize;
            return Some(AppEvent::SortByColumn(next));
        }
        if km.is_delete(code, mods) {
            return state.networks.items.get(state.networks.selected)
                .map(|n| AppEvent::ShowConfirmDialog(
                    format!("Remove network '{}'?", n.name),
                    ConfirmAction::RemoveNetwork(n.id.clone()),
                ));
        }
        None
    }
}
