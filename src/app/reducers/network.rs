use crate::app::event::{AppEvent, Command};
use crate::app::state::AppState;

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    let mut commands = Vec::new();
    match event {
        AppEvent::NetworksUpdated(networks) => {
            state.networks.update_items(networks.clone(), |_| true);
        }
        AppEvent::SelectNetwork(idx) if *idx < state.networks.items.len() => {
            state.networks.selected = *idx;
        }
        AppEvent::ToggleSortDirection => {
            state.networks.sort_ascending = !state.networks.sort_ascending;
            state.networks.apply_sort();
        }
        AppEvent::ToggleColumnPicker => {
            state.networks.show_column_picker = !state.networks.show_column_picker;
        }
        AppEvent::ToggleSelectionMode => {
            state.networks.selection_mode = !state.networks.selection_mode;
            if !state.networks.selection_mode {
                state.networks.selected_ids.clear();
            }
        }
        AppEvent::ToggleSelectResource(id) if state.networks.selection_mode => {
            if state.networks.selected_ids.contains(id) {
                state.networks.selected_ids.remove(id);
            } else {
                state.networks.selected_ids.insert(id.clone());
            }
        }
        AppEvent::SelectAllResources if state.networks.selection_mode => {
            for &idx in &state.networks.filtered {
                if let Some(n) = state.networks.items.get(idx) {
                    state.networks.selected_ids.insert(n.id.clone());
                }
            }
        }
        AppEvent::ToggleColumn(name) => {
            if state.networks.show_column_picker {
                let col_count = 5;
                crate::app::reducers::handle_column_nav(name, col_count, &mut state.networks.column_picker_selection);
                match name.as_str() {
                    "name" => state.config.network_columns.show_name = !state.config.network_columns.show_name,
                    "id" => state.config.network_columns.show_id = !state.config.network_columns.show_id,
                    "driver" => state.config.network_columns.show_driver = !state.config.network_columns.show_driver,
                    "scope" => state.config.network_columns.show_scope = !state.config.network_columns.show_scope,
                    "ipam" => state.config.network_columns.show_ipam = !state.config.network_columns.show_ipam,
                    _ => {}
                }
                commands.push(Command::SaveConfig);
            }
        }
        _ => {}
    }
    commands
}
