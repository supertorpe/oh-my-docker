use std::time::Instant;

use crate::app::event::{AppEvent, Command};
use crate::app::state::AppState;

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    let mut commands = Vec::new();
    match event {
        AppEvent::NetworksUpdated(networks) => {
            state.networks.items = networks.clone();
            state.networks.loading = false;
            state.networks.last_updated = Some(Instant::now());
        }
        AppEvent::SelectNetwork(idx) if *idx < state.networks.items.len() => {
            state.networks.selected = *idx;
        }
        AppEvent::ToggleColumnPicker => {
            state.networks.show_column_picker = !state.networks.show_column_picker;
        }
        AppEvent::ToggleColumn(name) => {
            if state.networks.show_column_picker {
                let col_count = 5;
                match name.as_str() {
                    "next" => state.networks.column_picker_selection = (state.networks.column_picker_selection + 1) % col_count,
                    "prev" => state.networks.column_picker_selection = (state.networks.column_picker_selection + col_count - 1) % col_count,
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
