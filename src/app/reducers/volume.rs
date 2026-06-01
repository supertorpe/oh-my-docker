use crate::app::event::{AppEvent, Command};
use crate::app::state::AppState;

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    let mut commands = Vec::new();
    match event {
        AppEvent::VolumesUpdated(volumes) => {
            state.volumes.update_items(volumes.clone(), |_| true);
        }
        AppEvent::SelectVolume(idx) if *idx < state.volumes.items.len() => {
            state.volumes.selected = *idx;
        }
        AppEvent::ToggleSortDirection => {
            state.volumes.sort_ascending = !state.volumes.sort_ascending;
            state.volumes.apply_sort();
        }
        AppEvent::ToggleColumnPicker => {
            state.volumes.show_column_picker = !state.volumes.show_column_picker;
        }
        AppEvent::ToggleSelectionMode => {
            state.volumes.selection_mode = !state.volumes.selection_mode;
            if !state.volumes.selection_mode {
                state.volumes.selected_ids.clear();
            }
        }
        AppEvent::ToggleSelectResource(id) if state.volumes.selection_mode => {
            if state.volumes.selected_ids.contains(id) {
                state.volumes.selected_ids.remove(id);
            } else {
                state.volumes.selected_ids.insert(id.clone());
            }
        }
        AppEvent::SelectAllResources if state.volumes.selection_mode => {
            for &idx in &state.volumes.filtered {
                if let Some(v) = state.volumes.items.get(idx) {
                    state.volumes.selected_ids.insert(v.name.clone());
                }
            }
        }
        AppEvent::ToggleColumn(name) => {
            if state.volumes.show_column_picker {
                let col_count = 4;
                crate::app::reducers::handle_column_nav(name, col_count, &mut state.volumes.column_picker_selection);
                match name.as_str() {
                    "name" => state.config.volume_columns.show_name = !state.config.volume_columns.show_name,
                    "driver" => state.config.volume_columns.show_driver = !state.config.volume_columns.show_driver,
                    "size" => state.config.volume_columns.show_size = !state.config.volume_columns.show_size,
                    "mountpoint" => state.config.volume_columns.show_mountpoint = !state.config.volume_columns.show_mountpoint,
                    _ => {}
                }
                commands.push(Command::SaveConfig);
            }
        }
        _ => {}
    }
    commands
}
