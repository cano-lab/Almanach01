use crate::api::{mount_directory, unmount_directory, exec_command, read_file, write_file, list_dir, TerminalSession, ExecResult, FileContent, DirEntry};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[function_component(TerminalPanel)]
pub fn terminal_panel() -> Html {
    let session = use_state(|| None::<TerminalSession>);
    let mount_path = use_state(|| "/root/.openclaw/workspace".to_string());
    let entries = use_state(Vec::<DirEntry>::new);
    let current_dir = use_state(|| ".".to_string());
    let selected_file = use_state(|| None::<FileContent>);
    let selected_file_path = use_state(|| String::new());
    let command = use_state(String::new);
    let command_output = use_state(|| None::<ExecResult>);
    let loading = use_state(|| false);
    let error = use_state(String::new);
    let editing = use_state(|| false);
    let edit_content = use_state(String::new);

    // Load directory listing
    let refresh_dir = {
        let session = session.clone();
        let current_dir = current_dir.clone();
        let entries = entries.clone();
        let error = error.clone();
        let loading = loading.clone();
        Callback::from(move |_| {
            if let Some(s) = (*session).clone() {
                let dir = (*current_dir).clone();
                let entries = entries.clone();
                let error = error.clone();
                let loading = loading.clone();
                spawn_local(async move {
                    loading.set(true);
                    match list_dir(&s.id, &dir).await {
                        Ok(data) => {
                            entries.set(data);
                            error.set(String::new());
                        }
                        Err(e) => error.set(e),
                    }
                    loading.set(false);
                });
            }
        })
    };

    let on_mount = {
        let mount_path = mount_path.clone();
        let session = session.clone();
        let error = error.clone();
        let loading = loading.clone();
        let refresh_dir = refresh_dir.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let path = (*mount_path).clone();
            let session = session.clone();
            let error = error.clone();
            let loading = loading.clone();
            let refresh_dir = refresh_dir.clone();
            spawn_local(async move {
                loading.set(true);
                match mount_directory(&path).await {
                    Ok(s) => {
                        session.set(Some(s));
                        error.set(String::new());
                        refresh_dir.emit(());
                    }
                    Err(e) => error.set(e),
                }
                loading.set(false);
            });
        })
    };

    let on_unmount = {
        let session = session.clone();
        let error = error.clone();
        Callback::from(move |_| {
            if let Some(s) = (*session).clone() {
                let session = session.clone();
                let error = error.clone();
                spawn_local(async move {
                    match unmount_directory(&s.id).await {
                        Ok(()) => session.set(None),
                        Err(e) => error.set(e),
                    }
                });
            }
        })
    };

    let on_path_input = {
        let mount_path = mount_path.clone();
        Callback::from(move |e: InputEvent| {
            let value = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
            mount_path.set(value);
        })
    };

    let on_command_input = {
        let command = command.clone();
        Callback::from(move |e: InputEvent| {
            let value = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
            command.set(value);
        })
    };

    let on_command_submit = {
        let session = session.clone();
        let command = command.clone();
        let command_output = command_output.clone();
        let error = error.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if let Some(s) = (*session).clone() {
                let cmd = (*command).clone();
                let command_output = command_output.clone();
                let error = error.clone();
                let command = command.clone();
                spawn_local(async move {
                    match exec_command(&s.id, &cmd).await {
                        Ok(result) => {
                            command_output.set(Some(result));
                            error.set(String::new());
                        }
                        Err(e) => error.set(e),
                    }
                    command.set(String::new());
                });
            }
        })
    };

    let on_navigate = {
        let current_dir = current_dir.clone();
        let refresh_dir = refresh_dir.clone();
        Callback::from(move |name: String| {
            let current_dir = current_dir.clone();
            let refresh_dir = refresh_dir.clone();
            let new_path = if (*current_dir) == "." {
                name
            } else {
                format!("{}/{}", *current_dir, name)
            };
            current_dir.set(new_path);
            refresh_dir.emit(());
        })
    };

    let on_navigate_up = {
        let current_dir = current_dir.clone();
        let refresh_dir = refresh_dir.clone();
        Callback::from(move |_| {
            let current_dir = current_dir.clone();
            let refresh_dir = refresh_dir.clone();
            let parts: Vec<&str> = (*current_dir).split('/').collect();
            if parts.len() > 1 {
                let new_path = parts[..parts.len()-1].join("/");
                current_dir.set(if new_path.is_empty() { ".".to_string() } else { new_path });
            } else {
                current_dir.set(".".to_string());
            }
            refresh_dir.emit(());
        })
    };

    let on_file_click = {
        let session = session.clone();
        let current_dir = current_dir.clone();
        let selected_file = selected_file.clone();
        let selected_file_path = selected_file_path.clone();
        let error = error.clone();
        let editing = editing.clone();
        let edit_content = edit_content.clone();
        Callback::from(move |(name, is_dir): (String, bool)| {
            if is_dir {
                return;
            }
            if let Some(s) = (*session).clone() {
                let path = if (*current_dir) == "." {
                    name.clone()
                } else {
                    format!("{}/{}", *current_dir, name)
                };
                let selected_file = selected_file.clone();
                let selected_file_path = selected_file_path.clone();
                let error = error.clone();
                let editing = editing.clone();
                let edit_content = edit_content.clone();
                spawn_local(async move {
                    match read_file(&s.id, &path).await {
                        Ok(content) => {
                            edit_content.set(content.content.clone());
                            selected_file.set(Some(content));
                            selected_file_path.set(path);
                            editing.set(false);
                            error.set(String::new());
                        }
                        Err(e) => error.set(e),
                    }
                });
            }
        })
    };

    let on_edit_toggle = {
        let editing = editing.clone();
        Callback::from(move |_| {
            editing.set(!*editing);
        })
    };

    let on_edit_content = {
        let edit_content = edit_content.clone();
        Callback::from(move |e: InputEvent| {
            let value = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
            edit_content.set(value);
        })
    };

    let on_save_file = {
        let session = session.clone();
        let selected_file_path = selected_file_path.clone();
        let edit_content = edit_content.clone();
        let error = error.clone();
        let editing = editing.clone();
        Callback::from(move |_| {
            if let Some(s) = (*session).clone() {
                let path = (*selected_file_path).clone();
                let content = (*edit_content).clone();
                let error = error.clone();
                let editing = editing.clone();
                spawn_local(async move {
                    match write_file(&s.id, &path, &content).await {
                        Ok(_) => {
                            editing.set(false);
                            error.set("File saved".to_string());
                        }
                        Err(e) => error.set(e),
                    }
                });
            }
        })
    };

    html! {
        <div class="terminal-panel">
            if let Some(s) = (*session).clone() {
                <div class="terminal-mounted">
                    <div class="terminal-header">
                        <div class="session-info">
                            <span class="badge badge-ok">{"MOUNTED"}</span>
                            <span class="session-path">{&s.path}</span>
                            <span class="session-id">{format!("({})", s.id)}</span>
                        </div>
                        <button class="btn-danger btn-sm" onclick={on_unmount}>{"Unmount"}</button>
                    </div>

                    if !error.is_empty() {
                        <div class="error-banner">{&*error}</div>
                    }

                    <div class="terminal-body">
                        <div class="file-explorer">
                            <div class="explorer-header">
                                <button class="btn-sm" onclick={on_navigate_up}>{"← Up"}</button>
                                <span class="current-dir">{&*current_dir}</span>
                                <button class="btn-sm" onclick={refresh_dir.reform(|_| ())}>{"Refresh"}</button>
                            </div>
                            if *loading {
                                <div class="loading">{"Loading..."}</div>
                            } else {
                                <div class="file-list">
                                    {entries.iter().map(|entry| {
                                        let name = entry.name.clone();
                                        let is_dir = entry.entry_type == "directory";
                                        let on_nav = {
                                            let on_navigate = on_navigate.clone();
                                            let name = name.clone();
                                            Callback::from(move |_| {
                                                on_navigate.emit(name.clone());
                                            })
                                        };
                                        let on_file = {
                                            let on_file_click = on_file_click.clone();
                                            let name = name.clone();
                                            Callback::from(move |_| {
                                                on_file_click.emit((name.clone(), is_dir));
                                            })
                                        };
                                        html! {
                                            <div class="file-entry">
                                                if is_dir {
                                                    <button class="file-dir" onclick={on_nav}>
                                                        {format!("📁 {}", entry.name)}
                                                    </button>
                                                } else {
                                                    <button class="file-file" onclick={on_file}>
                                                        {format!("📄 {} ({} bytes)", entry.name, entry.size)}
                                                    </button>
                                                }
                                            </div>
                                        }
                                    }).collect::<Html>()}
                                </div>
                            }
                        </div>

                        <div class="file-content">
                            if let Some(file) = (*selected_file).clone() {
                                <div class="file-header">
                                    <span>{&*selected_file_path}</span>
                                    <span>{format!("({} bytes)", file.size)}</span>
                                    <button class="btn-sm" onclick={on_edit_toggle}>
                                        {if *editing { "Cancel" } else { "Edit" }}
                                    </button>
                                </div>
                                if *editing {
                                    <textarea
                                        class="file-editor"
                                        value={(*edit_content).clone()}
                                        oninput={on_edit_content}
                                    />
                                    <button class="btn-primary" onclick={on_save_file}>{"Save"}</button>
                                } else {
                                    <pre class="file-view">{&file.content}</pre>
                                }
                            } else {
                                <div class="empty-state">{"Select a file to view"}</div>
                            }
                        </div>
                    </div>

                    <div class="terminal-console">
                        <form onsubmit={on_command_submit}>
                            <div class="command-row">
                                <span class="prompt">{"$"}</span>
                                <input
                                    type="text"
                                    value={(*command).clone()}
                                    oninput={on_command_input}
                                    placeholder="Enter command..."
                                    class="command-input"
                                />
                                <button type="submit" class="btn-primary">{"Run"}</button>
                            </div>
                        </form>
                        if let Some(output) = (*command_output).clone() {
                            <div class="command-output">
                                if !output.stdout.is_empty() {
                                    <pre class="stdout">{&output.stdout}</pre>
                                }
                                if !output.stderr.is_empty() {
                                    <pre class="stderr">{&output.stderr}</pre>
                                }
                                <div class="exit-code">{format!("Exit code: {}", output.exit_code)}</div>
                            </div>
                        }
                    </div>
                </div>
            } else {
                <div class="terminal-unmounted">
                    <h3>{"Mount a Directory"}</h3>
                    <p>{"Enter a directory path to mount it as a working session."}</p>
                    if !error.is_empty() {
                        <div class="error-banner">{&*error}</div>
                    }
                    <form onsubmit={on_mount}>
                        <div class="form-group">
                            <label>{"Directory Path"}</label>
                            <input
                                type="text"
                                value={(*mount_path).clone()}
                                oninput={on_path_input}
                                placeholder="/path/to/directory"
                            />
                        </div>
                        <button type="submit" class="btn-primary" disabled={*loading}>
                            {if *loading { "Mounting..." } else { "Mount" }}
                        </button>
                    </form>
                </div>
            }
        </div>
    }
}
