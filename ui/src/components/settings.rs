use crate::api::{fetch_api_keys, set_api_key, delete_api_key, ApiKeyInfo};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[function_component(SettingsPanel)]
pub fn settings_panel() -> Html {
    let keys = use_state(Vec::<ApiKeyInfo>::new);
    let loading = use_state(|| true);
    let error = use_state(String::new);
    let success = use_state(String::new);

    // Form state for adding a new key
    let new_provider = use_state(|| "augure".to_string());
    let new_key = use_state(String::new);
    let adding = use_state(|| false);

    // Load keys on mount
    {
        let keys = keys.clone();
        let loading = loading.clone();
        let error = error.clone();
        use_effect(move || {
            spawn_local(async move {
                loading.set(true);
                match fetch_api_keys().await {
                    Ok(data) => {
                        keys.set(data);
                        error.set(String::new());
                    }
                    Err(e) => {
                        error.set(e);
                    }
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_refresh = {
        let keys = keys.clone();
        let loading = loading.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let keys = keys.clone();
            let loading = loading.clone();
            let error = error.clone();
            spawn_local(async move {
                loading.set(true);
                match fetch_api_keys().await {
                    Ok(data) => {
                        keys.set(data);
                        error.set(String::new());
                    }
                    Err(e) => error.set(e),
                }
                loading.set(false);
            });
        })
    };

    let on_provider_input = {
        let new_provider = new_provider.clone();
        Callback::from(move |e: InputEvent| {
            let value = e
                .target_unchecked_into::<web_sys::HtmlInputElement>()
                .value();
            new_provider.set(value);
        })
    };

    let on_key_input = {
        let new_key = new_key.clone();
        Callback::from(move |e: InputEvent| {
            let value = e
                .target_unchecked_into::<web_sys::HtmlInputElement>()
                .value();
            new_key.set(value);
        })
    };

    let on_add = {
        let new_provider = new_provider.clone();
        let new_key = new_key.clone();
        let adding = adding.clone();
        let error = error.clone();
        let success = success.clone();
        let _keys = keys.clone();
        let on_refresh = on_refresh.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let provider = (*new_provider).clone();
            let key = (*new_key).clone();
            if provider.is_empty() || key.is_empty() {
                error.set("Provider and key are required".to_string());
                return;
            }
            adding.set(true);
            error.set(String::new());
            success.set(String::new());

            let success = success.clone();
            let adding = adding.clone();
            let error = error.clone();
            let on_refresh = on_refresh.clone();
            spawn_local(async move {
                match set_api_key(&provider, &key).await {
                    Ok(()) => {
                        success.set(format!("API key saved for {}", provider));
                        on_refresh.emit(());
                    }
                    Err(e) => error.set(e),
                }
                adding.set(false);
            });
        })
    };

    let on_delete = {
        let error = error.clone();
        let success = success.clone();
        let on_refresh = on_refresh.clone();
        Callback::from(move |provider: String| {
            let error = error.clone();
            let success = success.clone();
            let on_refresh = on_refresh.clone();
            spawn_local(async move {
                match delete_api_key(&provider).await {
                    Ok(()) => {
                        success.set(format!("API key removed for {}", provider));
                        on_refresh.emit(());
                    }
                    Err(e) => error.set(e),
                }
            });
        })
    };

    html! {
        <div class="settings-panel">
            <div class="settings-header">
                <h2>{"API Keys"}</h2>
                <button class="btn-secondary" onclick={on_refresh.reform(|_| ())} disabled={*loading}>
                    {if *loading { "Refreshing..." } else { "Refresh" }}
                </button>
            </div>

            if !error.is_empty() {
                <div class="error-banner">{&*error}</div>
            }
            if !success.is_empty() {
                <div class="success-banner">{&*success}</div>
            }

            <div class="key-list">
                if *loading {
                    <div class="loading">{"Loading API keys..."}</div>
                } else if keys.is_empty() {
                    <div class="empty-state">{"No API keys configured"}</div>
                } else {
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th>{"Provider"}</th>
                                <th>{"Status"}</th>
                                <th>{"Actions"}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {keys.iter().map(|k| {
                                let provider = k.provider.clone();
                                let on_delete_click = {
                                    let on_delete = on_delete.clone();
                                    let provider = provider.clone();
                                    Callback::from(move |_| {
                                        on_delete.emit(provider.clone());
                                    })
                                };
                                html! {
                                    <tr key={k.provider.clone()}>
                                        <td class="provider-name">{&k.provider}</td>
                                        <td>
                                            if k.has_key {
                                                <span class="badge badge-ok">{"Configured"}</span>
                                            } else {
                                                <span class="badge badge-missing">{"Not set"}</span>
                                            }
                                        </td>
                                        <td>
                                            <button
                                                class="btn-danger btn-sm"
                                                onclick={on_delete_click}
                                                disabled={!k.has_key}
                                            >
                                                {"Remove"}
                                            </button>
                                        </td>
                                    </tr>
                                }
                            }).collect::<Html>()}
                        </tbody>
                    </table>
                }
            </div>

            <div class="key-form-section">
                <h3>{"Add / Update API Key"}</h3>
                <form onsubmit={on_add}>
                    <div class="form-row">
                        <div class="form-group">
                            <label>{"Provider"}</label>
                            <input
                                type="text"
                                value={(*new_provider).clone()}
                                oninput={on_provider_input}
                                placeholder="e.g. augure, openai, kimi, anthropic, google"
                                disabled={*adding}
                            />
                        </div>
                        <div class="form-group flex-grow">
                            <label>{"API Key"}</label>
                            <input
                                type="password"
                                value={(*new_key).clone()}
                                oninput={on_key_input}
                                placeholder="sk-... or bearer token"
                                disabled={*adding}
                            />
                        </div>
                    </div>
                    <button type="submit" class="btn-primary" disabled={*adding}>
                        {if *adding { "Saving..." } else { "Save Key" }}
                    </button>
                </form>
            </div>

            <div class="help-text">
                <p>
                    {"Supported providers: "}
                    <code>{"augure"}</code>{" (Augure AI), "}
                    <code>{"kimi"}</code>{" (Moonshot), "}
                    <code>{"openai"}</code>{", "}
                    <code>{"anthropic"}</code>{", "}
                    <code>{"google"}</code>{" (Gemini), "}
                    <code>{"zai"}</code>{"."}
                </p>
                <p>{"Keys are stored server-side in data/api_keys.json and used for LLM calls."}</p>
            </div>
        </div>
    }
}
