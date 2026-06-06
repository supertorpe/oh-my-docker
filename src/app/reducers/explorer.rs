use std::path::PathBuf;

use crate::app::event::{AppEvent, Command};
use crate::app::mode::Mode;
use crate::app::state::{AppState, ExplorerEntry};
use crate::search::fuzzy::Fuzzy;

fn apply_filter(items: &[ExplorerEntry], filter: &str) -> Vec<ExplorerEntry> {
    if filter.is_empty() {
        items.to_vec()
    } else {
        let fuzzy = Fuzzy::new();
        items.iter()
            .filter(|e| fuzzy.score(filter, &e.name).is_some())
            .cloned()
            .collect()
    }
}

fn host_entry_index(state: &AppState) -> Option<usize> {
    let show_parent = state.explorer.host.path != "/";
    let idx = if show_parent { state.explorer.host.selected.saturating_sub(1) } else { state.explorer.host.selected };
    if idx < state.explorer.host.items.len() { Some(idx) } else { None }
}

fn container_entry_index(state: &AppState) -> Option<usize> {
    let show_parent = state.explorer.container.path != "/";
    let idx = if show_parent { state.explorer.container.selected.saturating_sub(1) } else { state.explorer.container.selected };
    if idx < state.explorer.container.items.len() { Some(idx) } else { None }
}

fn volume_name(state: &AppState) -> Option<String> {
    if let Mode::ExplorerVolume(_, name) = state.navigation.mode_stack.current() {
        Some(name.clone())
    } else {
        None
    }
}

fn list_cmd(path: &str) -> Command {
    Command::ListHostDir(path.to_string())
}

fn list_container_cmd(state: &AppState, path: &str) -> Command {
    if let Some(name) = volume_name(state) {
        Command::ListVolumeDir(name, path.to_string())
    } else {
        Command::ListContainerDir(state.explorer.container_id.clone(), path.to_string())
    }
}

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    let mut commands = Vec::new();

    match event {
        AppEvent::ExplorerHostToggleSelect => {
            let panel = &mut state.explorer.host;
            let show_parent = panel.path != "/";
            let entry_idx = if show_parent { panel.selected.saturating_sub(1) } else { panel.selected };
            if let Some(entry) = panel.items.get(entry_idx) {
                if panel.selected_names.contains(&entry.name) {
                    panel.selected_names.remove(&entry.name);
                } else {
                    panel.selected_names.insert(entry.name.clone());
                }
            }
        }
        AppEvent::ExplorerContainerToggleSelect => {
            let panel = &mut state.explorer.container;
            let show_parent = panel.path != "/";
            let entry_idx = if show_parent { panel.selected.saturating_sub(1) } else { panel.selected };
            if let Some(entry) = panel.items.get(entry_idx) {
                if panel.selected_names.contains(&entry.name) {
                    panel.selected_names.remove(&entry.name);
                } else {
                    panel.selected_names.insert(entry.name.clone());
                }
            }
        }
        AppEvent::ExplorerCopySelected => {
            let host_sel = &state.explorer.host.selected_names;
            let container_sel = &state.explorer.container.selected_names;
            let sel = if !host_sel.is_empty() { &state.explorer.host } else if !container_sel.is_empty() { &state.explorer.container } else { return commands; };
            let is_host = !host_sel.is_empty();
            for name in &sel.selected_names {
                let full_path = if sel.path == "/" || sel.path.is_empty() {
                    format!("/{}", name)
                } else {
                    let p = sel.path.strip_suffix('/').unwrap_or(&sel.path);
                    format!("{}/{}", p, name)
                };
                if is_host {
                    let container_dest = if state.explorer.container.path.ends_with('/') {
                        state.explorer.container.path.clone()
                    } else {
                        format!("{}/", state.explorer.container.path)
                    };
                    commands.push(Command::CopyToContainer(
                        state.explorer.container_id.clone(),
                        full_path,
                        container_dest,
                    ));
                } else {
                    let host_dest = state.explorer.host.path.clone();
                    commands.push(Command::CopyFromContainer(
                        state.explorer.container_id.clone(),
                        full_path,
                        host_dest,
                    ));
                }
            }
        }
        AppEvent::ExplorerDeleteSelected => {
            let host_sel = &state.explorer.host.selected_names;
            let container_sel = &state.explorer.container.selected_names;
            if !host_sel.is_empty() {
                let path = &state.explorer.host.path;
                for name in host_sel {
                    let full_path = if path == "/" || path.is_empty() {
                        format!("/{}", name)
                    } else {
                        format!("{}/{}", path, name)
                    };
                    commands.push(Command::DeleteHostFile(full_path));
                }
            } else if !container_sel.is_empty() {
                let path = &state.explorer.container.path;
                let cid = &state.explorer.container_id;
                for name in container_sel {
                    let full_path = if path == "/" || path.is_empty() {
                        format!("/{}", name)
                    } else {
                        let p = path.strip_suffix('/').unwrap_or(path);
                        format!("{}/{}", p, name)
                    };
                    commands.push(Command::DeleteContainerFile(cid.clone(), full_path));
                }
            }
        }
        AppEvent::ExplorerSelect => {
            state.explorer.focus = match state.explorer.focus {
                crate::app::state::ExplorerFocus::Left => crate::app::state::ExplorerFocus::Right,
                crate::app::state::ExplorerFocus::Right => crate::app::state::ExplorerFocus::Left,
            };
        }

        AppEvent::ExplorerHostSelect(idx) => {
            state.explorer.host.selected = *idx;
        }

        AppEvent::ExplorerContainerSelect(idx) => {
            state.explorer.container.selected = *idx;
        }

        AppEvent::ExplorerHostGoUp => {
            if let Some(parent) = PathBuf::from(&state.explorer.host.path).parent()
                .and_then(|p| p.to_str())
                .filter(|s| !s.is_empty())
            {
                state.explorer.host.path = if parent == "/" { "/".to_string() } else { parent.to_string() };
                state.explorer.host.selected = 0;
                state.explorer.host.filter = String::new();
                state.explorer.host.filter_active = false;
                state.explorer.host.rename_active = false;
                state.explorer.host.rename_buffer = String::new();
                commands.push(list_cmd( &state.explorer.host.path));
            }
        }

        AppEvent::ExplorerContainerGoUp => {
            let path = &state.explorer.container.path;
            if path == "/" {
                return commands;
            }
            let cleaned = path.strip_suffix('/').unwrap_or(path);
            let parts: Vec<&str> = cleaned.split('/').filter(|s| !s.is_empty()).collect();
            if parts.len() <= 1 {
                state.explorer.container.path = "/".to_string();
                state.explorer.container.selected = 0;
                state.explorer.container.filter = String::new();
                state.explorer.container.filter_active = false;
                state.explorer.container.rename_active = false;
                state.explorer.container.rename_buffer = String::new();
                commands.push(list_container_cmd(state, "/"));
                return commands;
            }
            let parent = format!("/{}/", parts[..parts.len() - 1].join("/"));
            let parent_clone = parent.clone();
            state.explorer.container.path = parent;
            state.explorer.container.selected = 0;
            state.explorer.container.filter = String::new();
            state.explorer.container.filter_active = false;
            state.explorer.container.rename_active = false;
            state.explorer.container.rename_buffer = String::new();
            commands.push(list_container_cmd(state, &parent_clone));
        }

        AppEvent::ExplorerHostEnterDir(name) => {
            let new_path = if state.explorer.host.path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", state.explorer.host.path, name)
            };
            let new_path_clone = new_path.clone();
            state.explorer.host.path = new_path;
            state.explorer.host.selected = 0;
            state.explorer.host.filter = String::new();
            state.explorer.host.filter_active = false;
            state.explorer.host.rename_active = false;
            state.explorer.host.rename_buffer = String::new();
            commands.push(list_cmd( &new_path_clone));
        }

        AppEvent::ExplorerContainerEnterDir(name) => {
            let path = &state.explorer.container.path;
            let new_path = if path.ends_with('/') {
                format!("{}{}", path, name)
            } else {
                format!("{}/{}", path, name)
            };
            state.explorer.container.path = format!("{}/", new_path);
            state.explorer.container.selected = 0;
            state.explorer.container.filter = String::new();
            state.explorer.container.filter_active = false;
            state.explorer.container.rename_active = false;
            state.explorer.container.rename_buffer = String::new();
            commands.push(list_container_cmd(state, &state.explorer.container.path));
        }

        AppEvent::ExplorerHostRefresh => {
            commands.push(list_cmd( &state.explorer.host.path));
        }

        AppEvent::ExplorerContainerRefresh => {
            commands.push(list_container_cmd(state, &state.explorer.container.path));
        }

        AppEvent::ExplorerHostActivateFilter => {
            state.explorer.host.filter_active = true;
        }

        AppEvent::ExplorerContainerActivateFilter => {
            state.explorer.container.filter_active = true;
        }

        AppEvent::ExplorerFilter(q) => {
            if state.explorer.host.filter_active {
                state.explorer.host.filter = q.clone();
                if !q.is_empty() {
                    state.explorer.host.items = apply_filter(&state.explorer.host.all_items, q);
                }
            } else if state.explorer.container.filter_active {
                state.explorer.container.filter = q.clone();
                if !q.is_empty() {
                    state.explorer.container.items = apply_filter(&state.explorer.container.all_items, q);
                }
            }
            if q.is_empty() {
                state.explorer.host.filter_active = false;
                state.explorer.container.filter_active = false;
                state.explorer.host.items = state.explorer.host.all_items.clone();
                state.explorer.container.items = state.explorer.container.all_items.clone();
            }
        }

        AppEvent::ExplorerFilterSubmit => {
            state.explorer.host.filter_active = false;
            state.explorer.container.filter_active = false;
        }

        AppEvent::ExplorerHostActivateRename => {
            let panel = &mut state.explorer.host;
            let show_parent = panel.path != "/";
            let entry_idx = if show_parent { panel.selected.saturating_sub(1) } else { panel.selected };
            if let Some(entry) = panel.items.get(entry_idx) {
                panel.rename_buffer = entry.name.clone();
                panel.rename_active = true;
            }
        }

        AppEvent::ExplorerContainerActivateRename => {
            let panel = &mut state.explorer.container;
            let show_parent = panel.path != "/";
            let entry_idx = if show_parent { panel.selected.saturating_sub(1) } else { panel.selected };
            if let Some(entry) = panel.items.get(entry_idx) {
                panel.rename_buffer = entry.name.clone();
                panel.rename_active = true;
            }
        }

        AppEvent::ExplorerRenameUpdate(q) => {
            if state.explorer.host.rename_active {
                state.explorer.host.rename_buffer = q.clone();
            } else if state.explorer.container.rename_active {
                state.explorer.container.rename_buffer = q.clone();
            }
        }

        AppEvent::ExplorerRenameCancel => {
            if state.explorer.host.rename_active {
                state.explorer.host.rename_active = false;
                state.explorer.host.rename_buffer = String::new();
            }
            if state.explorer.container.rename_active {
                state.explorer.container.rename_active = false;
                state.explorer.container.rename_buffer = String::new();
            }
        }

        AppEvent::ExplorerHostActivateCreate => {
            state.explorer.host.create_buffer = String::new();
            state.explorer.host.create_active = true;
        }
        AppEvent::ExplorerContainerActivateCreate => {
            state.explorer.container.create_buffer = String::new();
            state.explorer.container.create_active = true;
        }
        AppEvent::ExplorerCreateUpdate(q) => {
            if state.explorer.host.create_active {
                state.explorer.host.create_buffer = q.clone();
            } else if state.explorer.container.create_active {
                state.explorer.container.create_buffer = q.clone();
            }
        }
        AppEvent::ExplorerCreateCancel => {
            if state.explorer.host.create_active {
                state.explorer.host.create_active = false;
                state.explorer.host.create_buffer = String::new();
            }
            if state.explorer.container.create_active {
                state.explorer.container.create_active = false;
                state.explorer.container.create_buffer = String::new();
            }
        }
        AppEvent::ExplorerCreateSubmit => {
            if state.explorer.host.create_active {
                let name = std::mem::take(&mut state.explorer.host.create_buffer);
                state.explorer.host.create_active = false;
                if !name.is_empty() {
                    let (name, is_dir) = if name.ends_with('/') {
                        (name.strip_suffix('/').unwrap_or(&name).to_string(), true)
                    } else {
                        (name, false)
                    };
                    let full_path = if state.explorer.host.path == "/" {
                        format!("/{}", name)
                    } else {
                        format!("{}/{}", state.explorer.host.path, name)
                    };
                    commands.push(Command::CreateHostFile(full_path, is_dir));
                }
            } else if state.explorer.container.create_active {
                let name = std::mem::take(&mut state.explorer.container.create_buffer);
                state.explorer.container.create_active = false;
                if !name.is_empty() {
                    let (name, is_dir) = if name.ends_with('/') {
                        (name.strip_suffix('/').unwrap_or(&name).to_string(), true)
                    } else {
                        (name, false)
                    };
                    let full_path = if state.explorer.container.path == "/" {
                        format!("/{}", name)
                    } else {
                        let p = state.explorer.container.path.strip_suffix('/').unwrap_or(&state.explorer.container.path);
                        format!("{}/{}", p, name)
                    };
                    commands.push(Command::CreateContainerFile(
                        state.explorer.container_id.clone(),
                        full_path,
                        is_dir,
                    ));
                }
            }
        }
        AppEvent::ExplorerHostActivateGoto => {
            state.explorer.host.goto_buffer = state.explorer.host.path.clone();
            state.explorer.host.goto_active = true;
        }
        AppEvent::ExplorerContainerActivateGoto => {
            state.explorer.container.goto_buffer = state.explorer.container.path.clone();
            state.explorer.container.goto_active = true;
        }
        AppEvent::ExplorerGotoUpdate(q) => {
            if state.explorer.host.goto_active {
                state.explorer.host.goto_buffer = q.clone();
            } else if state.explorer.container.goto_active {
                state.explorer.container.goto_buffer = q.clone();
            }
        }
        AppEvent::ExplorerGotoCancel => {
            if state.explorer.host.goto_active {
                state.explorer.host.goto_active = false;
                state.explorer.host.goto_buffer = String::new();
            }
            if state.explorer.container.goto_active {
                state.explorer.container.goto_active = false;
                state.explorer.container.goto_buffer = String::new();
            }
        }
        AppEvent::ExplorerGotoSubmit => {
            if state.explorer.host.goto_active {
                let path = std::mem::take(&mut state.explorer.host.goto_buffer);
                state.explorer.host.goto_active = false;
                let path = if path.is_empty() { "/".to_string() } else { path };
                state.explorer.host.path = path.clone();
                state.explorer.host.selected = 0;
                state.explorer.host.filter = String::new();
                state.explorer.host.filter_active = false;
                state.explorer.host.rename_active = false;
                state.explorer.host.rename_buffer = String::new();
                state.explorer.host.loading = true;
                commands.push(list_cmd(&path));
            } else if state.explorer.container.goto_active {
                let path = std::mem::take(&mut state.explorer.container.goto_buffer);
                state.explorer.container.goto_active = false;
                let path = if path.is_empty() { "/".to_string() } else { path };
                let path = if path.ends_with('/') { path } else { format!("{}/", path) };
                state.explorer.container.path = path.clone();
                state.explorer.container.selected = 0;
                state.explorer.container.filter = String::new();
                state.explorer.container.filter_active = false;
                state.explorer.container.rename_active = false;
                state.explorer.container.rename_buffer = String::new();
                state.explorer.container.loading = true;
                commands.push(list_container_cmd(state, &path));
            }
        }
        AppEvent::ExplorerRenameSubmit => {
            if state.explorer.host.rename_active {
                let panel = &mut state.explorer.host;
                let show_parent = panel.path != "/";
                let entry_idx = if show_parent { panel.selected.saturating_sub(1) } else { panel.selected };
                if let Some(entry) = panel.items.get(entry_idx) {
                    if !panel.rename_buffer.is_empty() && panel.rename_buffer != entry.name {
                        let old_path = if panel.path == "/" {
                            format!("/{}", entry.name)
                        } else {
                            format!("{}/{}", panel.path, entry.name)
                        };
                        let new_path = if panel.path == "/" {
                            format!("/{}", panel.rename_buffer)
                        } else {
                            format!("{}/{}", panel.path, panel.rename_buffer)
                        };
                        commands.push(Command::RenameHostFile(old_path, new_path));
                    }
                }
                panel.rename_active = false;
                panel.rename_buffer = String::new();
            } else if state.explorer.container.rename_active {
                let panel = &mut state.explorer.container;
                let show_parent = panel.path != "/";
                let entry_idx = if show_parent { panel.selected.saturating_sub(1) } else { panel.selected };
                if let Some(entry) = panel.items.get(entry_idx) {
                    if !panel.rename_buffer.is_empty() && panel.rename_buffer != entry.name {
                        let old_path = if panel.path == "/" {
                            format!("/{}", entry.name)
                        } else {
                            format!("{}/{}", panel.path, entry.name)
                        };
                        let new_path = if panel.path == "/" {
                            format!("/{}", panel.rename_buffer)
                        } else {
                            format!("{}/{}", panel.path, panel.rename_buffer)
                        };
                        commands.push(Command::RenameContainerFile(
                            state.explorer.container_id.clone(),
                            old_path,
                            new_path,
                        ));
                    }
                }
                panel.rename_active = false;
                panel.rename_buffer = String::new();
            }
        }
        AppEvent::ExplorerContainerDirUpdated(container_id, path, entries) => {
            if state.explorer.container_id == *container_id
                && state.explorer.container.path == *path
            {
                state.explorer.container.all_items = entries.clone();
                let filter = &state.explorer.container.filter;
                state.explorer.container.items = apply_filter(entries, filter);
                state.explorer.container.selected = 0;
                state.explorer.container.loading = false;
            }
        }

        AppEvent::ContainerWorkingDir(container_id, working_dir) => {
            if state.explorer.container_id == *container_id {
                let path = if working_dir.is_empty() || !working_dir.starts_with('/') {
                    "/".to_string()
                } else if working_dir.ends_with('/') {
                    working_dir.clone()
                } else {
                    format!("{}/", working_dir)
                };
                state.explorer.container.path = path.clone();
                state.explorer.container.selected = 0;
                state.explorer.container.loading = true;
                commands.push(Command::ListContainerDir(container_id.clone(), path));
            }
        }

        AppEvent::ExplorerHostDirUpdated(path, entries) => {
            if state.explorer.host.path == *path {
                state.explorer.host.all_items = entries.clone();
                let filter = &state.explorer.host.filter;
                state.explorer.host.items = apply_filter(entries, filter);
                state.explorer.host.selected = 0;
                state.explorer.host.loading = false;
            }
        }

        AppEvent::ExplorerCopyToContainer => {
            if let Some(entry_idx) = host_entry_index(state) {
                if let Some(entry) = state.explorer.host.items.get(entry_idx) {
                    let host_path = if state.explorer.host.path == "/" {
                        format!("/{}", entry.name)
                    } else {
                        format!("{}/{}", state.explorer.host.path, entry.name)
                    };
                    let container_dest = if state.explorer.container.path.ends_with('/') {
                        state.explorer.container.path.clone()
                    } else {
                        format!("{}/", state.explorer.container.path)
                    };
                    state.explorer.transfer_in_progress = true;
                    state.explorer.transfer_message = Some(format!("Copying {}...", entry.name));
                    state.explorer.transfer_error = None;
                    commands.push(Command::CopyToContainer(
                        state.explorer.container_id.clone(),
                        host_path,
                        container_dest,
                    ));
                }
            }
        }

        AppEvent::ExplorerCopyFromContainer => {
            if let Some(entry_idx) = container_entry_index(state) {
                if let Some(entry) = state.explorer.container.items.get(entry_idx) {
                    let container_src = if state.explorer.container.path.ends_with('/') {
                        format!("{}{}", state.explorer.container.path, entry.name)
                    } else {
                        format!("{}/{}", state.explorer.container.path, entry.name)
                    };
                    let host_dest = state.explorer.host.path.clone();
                    state.explorer.transfer_in_progress = true;
                    state.explorer.transfer_message = Some(format!("Copying {}...", entry.name));
                    state.explorer.transfer_error = None;
                    commands.push(Command::CopyFromContainer(
                        state.explorer.container_id.clone(),
                        container_src,
                        host_dest,
                    ));
                }
            }
        }

        AppEvent::ExplorerTransferComplete(msg) => {
            state.explorer.transfer_in_progress = false;
            state.explorer.transfer_message = Some(msg.clone());
            state.explorer.transfer_error = None;
            state.explorer.transfer_message_clear_tick = state.tick_count + 2;
            commands.push(list_cmd( &state.explorer.host.path));
            commands.push(Command::ListContainerDir(
                state.explorer.container_id.clone(),
                state.explorer.container.path.clone(),
            ));
        }

        AppEvent::ExplorerTransferError(msg) => {
            state.explorer.transfer_in_progress = false;
            state.explorer.transfer_error = Some(msg.clone());
            state.explorer.transfer_message = None;
            state.explorer.transfer_error_clear_tick = state.tick_count + 2;
        }

        _ => {}
    }

    commands
}
