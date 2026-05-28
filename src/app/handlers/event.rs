use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::event::AppEvent;
use crate::app::state::AppState;

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if state.events.filter_active {
        match (key.code, key.modifiers) {
            (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                let new_q = state.events.filter.chars().take(state.events.filter.chars().count().saturating_sub(1)).collect();
                Some(AppEvent::FilterEvents(new_q))
            }
            (KeyCode::Esc, _) => Some(AppEvent::FilterEvents(String::new())),
            (KeyCode::Enter, _) => Some(AppEvent::EventsFilterSubmit),
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                let new_q = format!("{}{}", state.events.filter, c);
                Some(AppEvent::FilterEvents(new_q))
            }
            _ => None,
        }
    } else {
        let km = &state.keymap;
        let code = key.code;
        let mods = key.modifiers;

        if km.is_jump_top(code, mods) {
            return Some(AppEvent::JumpTop);
        }
        if km.is_jump_bottom(code, mods) {
            return Some(AppEvent::JumpBottom);
        }
        if km.is_navigate_up(code, mods) || code == KeyCode::Up {
            return Some(AppEvent::ScrollEvents(1));
        }
        if km.is_navigate_down(code, mods) || code == KeyCode::Down {
            return Some(AppEvent::ScrollEvents(-1));
        }
        if code == KeyCode::PageUp {
            return Some(AppEvent::ScrollEvents(20));
        }
        if code == KeyCode::PageDown {
            return Some(AppEvent::ScrollEvents(-20));
        }
        if km.is_search(code, mods) {
            return Some(AppEvent::ActivateEventsFilter);
        }
        None
    }
}
