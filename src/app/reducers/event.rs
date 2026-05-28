use std::time::Instant;

use crate::app::event::{AppEvent, Command};
use crate::app::state::AppState;

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    let commands = Vec::new();
    match event {
        AppEvent::EventsUpdated(events) => {
            state.events.buffer.extend(events.iter().cloned());
            if state.events.buffer.len() > state.events.max_events {
                state.events.buffer.truncate(state.events.max_events);
            }
            state.events.last_updated = Some(Instant::now());
        }
        AppEvent::ActivateEventsFilter => {
            state.events.filter_active = true;
        }
        AppEvent::EventsFilterSubmit => {
            state.events.filter_active = false;
        }
        AppEvent::FilterEvents(q) => {
            state.events.filter = q.clone();
            if state.events.filter.is_empty() {
                state.events.filter_active = false;
            }
        }
        AppEvent::ScrollEvents(delta) => {
            let max_offset = state.events.buffer.len().saturating_sub(state.events.viewport_height);
            state.events.scroll_offset = crate::util::scroll_offset(state.events.scroll_offset, *delta, max_offset);
        }
        _ => {}
    }
    commands
}
