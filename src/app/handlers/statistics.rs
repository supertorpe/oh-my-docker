use crossterm::event::{KeyCode, KeyEvent};
use crate::app::event::AppEvent;
use crate::app::state::AppState;

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    let km = &state.keymap;
    let code = key.code;
    let mods = key.modifiers;

    if code == KeyCode::Esc {
        return Some(AppEvent::Back);
    }
    if km.is_statistics_sort(code, mods) {
        return Some(AppEvent::CycleSortStat(1));
    }
    if km.is_statistics_sort_desc(code, mods) {
        return Some(AppEvent::ToggleSortDirection);
    }
    if code == KeyCode::Left {
        return Some(AppEvent::CycleSortStat(-1));
    }
    if code == KeyCode::Right {
        return Some(AppEvent::CycleSortStat(1));
    }
    None
}
