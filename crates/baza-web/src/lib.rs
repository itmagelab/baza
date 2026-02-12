use leptos::task::spawn_local;
use leptos::prelude::*;
use leptos::either::Either;

#[derive(Clone, Copy, Debug, PartialEq)]
enum AppView {
    Login,
    Dashboard,
    AddBundle,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct BundleInfo {
    name: String,
}

#[component]
pub fn App() -> impl IntoView {
    let (view, set_view) = signal(AppView::Login);
    let (passphrase, set_passphrase) = signal(String::new());
    let (bundles, set_bundles) = signal(Vec::<BundleInfo>::new());
    let (search_query, set_search_query) = signal(String::new());
    let (error_msg, set_error_msg) = signal(String::new());
    let (show_init_confirm, set_show_init_confirm) = signal(false);
    let (init_passphrase, set_init_passphrase) = signal(None::<String>);

    // Add Bundle Form State
    let (new_bundle_name, set_new_bundle_name) = signal(String::new());
    let (new_bundle_pass, set_new_bundle_pass) = signal(String::new());
    let (is_editing, set_is_editing) = signal(false);
    let (show_delete_confirm, set_show_delete_confirm) = signal(false);

    let load_bundles = move || {
        spawn_local(async move {
            match baza_core::storage::list_all_keys().await {
                Ok(keys) => {
                    let info = keys.into_iter()
                        .map(|name| BundleInfo { name })
                        .collect();
                    set_bundles.set(info);
                }
                Err(e) => set_error_msg.set(format!("Load failed: {}", e)),
            }
        });
    };

    // Actions
    let perform_unlock = move || {
        let p = passphrase.get();
        if p.is_empty() {
            set_error_msg.set("Passphrase cannot be empty".to_string());
            return;
        }
        match baza_core::unlock(Some(p)) {
            Ok(_) => {
                set_init_passphrase.set(None); // Clear init key if logging in normally
                set_view.set(AppView::Dashboard);
                set_error_msg.set(String::new());
                load_bundles();
            }
            Err(e) => set_error_msg.set(format!("Unlock failed: {}", e)),
        }
    };

    let perform_init = move |force: bool| {
        spawn_local(async move {
            if !force {
                match baza_core::storage::is_initialized().await {
                    Ok(true) => {
                        set_show_init_confirm.set(true);
                        return;
                    }
                    Err(e) => {
                        set_error_msg.set(format!("Check failed: {}", e));
                        return;
                    }
                    Ok(false) => {}
                }
            }

            let p = if passphrase.get().is_empty() {
                None
            } else {
                Some(passphrase.get())
            };

            match baza_core::init(p) {
                Ok(key) => {
                    set_init_passphrase.set(Some(key));
                    set_show_init_confirm.set(false);
                    set_view.set(AppView::Dashboard);
                    set_error_msg.set(String::new());
                    set_bundles.set(vec![]);
                }
                Err(e) => set_error_msg.set(format!("Init failed: {}", e)),
            }
        });
    };

    let perform_lock = move || {
        let _ = baza_core::lock();
        set_view.set(AppView::Login);
        set_passphrase.set(String::new());
    };

    let perform_save_bundle = move || {
        let name = new_bundle_name.get();
        let pass = new_bundle_pass.get();
        if name.is_empty() || pass.is_empty() {
            set_error_msg.set("Name and content are required".to_string());
            return;
        }
        
        spawn_local(async move {
            match baza_core::container::add(name.clone(), Some(pass)).await {
                Ok(_) => {
                    set_view.set(AppView::Dashboard);
                    set_new_bundle_name.set(String::new());
                    set_new_bundle_pass.set(String::new());
                    set_is_editing.set(false);
                    set_error_msg.set(String::new());
                    load_bundles();
                }
                Err(e) => set_error_msg.set(format!("Save failed: {}", e)),
            }
        });
    };

    let perform_edit = move |name: String| {
        let name_clone = name.clone();
        spawn_local(async move {
            match baza_core::storage::get_content(name_clone.clone()).await {
                Ok(content) => {
                    set_new_bundle_name.set(name_clone);
                    set_new_bundle_pass.set(content);
                    set_is_editing.set(true);
                    set_show_delete_confirm.set(false);
                    set_view.set(AppView::AddBundle);
                }
                Err(e) => set_error_msg.set(format!("Load failed: {}", e)),
            }
        });
    };

    let perform_delete = move || {
        let name = new_bundle_name.get();
        spawn_local(async move {
            match baza_core::storage::delete_by_name(name).await {
                Ok(_) => {
                    set_show_delete_confirm.set(false);
                    set_view.set(AppView::Dashboard);
                    set_new_bundle_name.set(String::new());
                    set_new_bundle_pass.set(String::new());
                    load_bundles();
                }
                Err(e) => set_error_msg.set(format!("Delete failed: {}", e)),
            }
        });
    };

    let perform_copy_first_line = move |name: String| {
        spawn_local(async move {
            match baza_core::storage::get_content(name).await {
                Ok(content) => {
                    let first_line = content.lines().next().unwrap_or("");
                    // We need a way to copy to clipboard in browser
                    // Leptos/web_sys approach:
                    if let Some(window) = web_sys::window() {
                        let _ = window.navigator().clipboard().write_text(first_line);
                        set_error_msg.set("COPIED TO CLIPBOARD".to_string());
                        // Clear message after 2 seconds
                        spawn_local(async move {
                            gloo_timers::future::TimeoutFuture::new(2000).await;
                            set_error_msg.set(String::new());
                        });
                    }
                }
                Err(e) => set_error_msg.set(format!("Copy failed: {}", e)),
            }
        });
    };

    let generate_password = move |_| {
        if let Ok(p) = baza_core::generate(24, false, false, false) {
            set_new_bundle_pass.set(p);
        }
    };

    // Auto-load on mount if already in Dashboard? 
    // App starts in Login, so we only load after success.

    let filtered_bundles = move || {
        let query = search_query.get().to_lowercase();
        bundles.get().into_iter()
            .filter(|b| b.name.to_lowercase().contains(&query))
            .collect::<Vec<_>>()
    };

    view! {
        <div class="container">
            <h1>"BAZA"</h1>

            {move || match view.get() {
                AppView::Login => Either::Left(view! {
                    <div class="view-login">
                        {move || show_init_confirm.get().then(|| view! {
                            <div class="confirm-box">
                                <p class="warning">"VAULT EXISTS! OVERWRITE?"</p>
                                <button class="btn btn-danger" on:click=move |_| perform_init(true)>"CONTINUE"</button>
                                <button class="btn btn-ghost" on:click=move |_| set_show_init_confirm.set(false)>"CANCEL"</button>
                            </div>
                        })}

                        <div style:display=move || if show_init_confirm.get() { "none" } else { "block" }>
                            <div class="form-group">
                                <label>"PASSPHRASE"</label>
                                <input
                                    type="password"
                                    placeholder="Enter your passphrase"
                                    prop:value=passphrase
                                    on:input=move |ev| set_passphrase.set(event_target_value(&ev))
                                    on:keydown=move |ev| if ev.key() == "Enter" { perform_unlock(); }
                                />
                            </div>
                            <button class="btn" on:click=move |_| perform_unlock()>"UNLOCK"</button>
                            <button class="btn btn-ghost" on:click=move |_| perform_init(false)>"INITIALIZE NEW VAULT"</button>
                            
                            {move || {
                                let msg = error_msg.get();
                                (!msg.is_empty()).then(|| view! { <p class="error">{msg}</p> })
                            }}
                        </div>
                    </div>
                }),

                AppView::Dashboard => Either::Right(Either::Left(view! {
                    <div class="view-dashboard">
                        {move || init_passphrase.get().map(|p| view! {
                            <div class="passphrase-banner">
                                <p>"YOUR NEW PASSPHRASE:"</p>
                                <code>{p}</code>
                                <p class="small">"SAVE IT NOW. IT WILL NOT BE SHOWN AGAIN."</p>
                            </div>
                        })}

                        <div class="search-group">
                            <input
                                type="text"
                                placeholder="Search bundles..."
                                prop:value=search_query
                                on:input=move |ev| set_search_query.set(event_target_value(&ev))
                            />
                            {move || (!search_query.get().is_empty()).then(|| view! {
                                <button class="search-clear" on:click=move |_| set_search_query.set(String::new())>"×"</button>
                            })}
                        </div>

                        <ul class="bundle-list">
                            <For
                                each=filtered_bundles
                                key=|b| b.name.clone()
                                children=move |b| {
                                    let name = b.name.clone();
                                    let name_for_edit = name.clone();
                                    let name_for_copy = name.clone();
                                    let name_for_click = name.clone();
                                    view! {
                                        <li class="bundle-item" on:click=move |_| perform_edit(name_for_click.clone())>
                                            <span class="bundle-name">{name}</span>
                                            <div class="bundle-actions">
                                                <button class="action-btn" title="Copy First Line" on:click=move |ev| {
                                                    ev.stop_propagation();
                                                    perform_copy_first_line(name_for_copy.clone());
                                                }>"📋"</button>
                                                <button class="action-btn" title="Edit" on:click=move |ev| {
                                                    ev.stop_propagation();
                                                    perform_edit(name_for_edit.clone());
                                                }>"✏️"</button>
                                            </div>
                                        </li>
                                    }
                                }
                            />
                        </ul>

                        <button class="btn" on:click=move |_| {
                            set_is_editing.set(false);
                            set_show_delete_confirm.set(false);
                            set_new_bundle_name.set(String::new());
                            set_new_bundle_pass.set(String::new());
                            set_view.set(AppView::AddBundle);
                        }>"ADD NEW BUNDLE"</button>
                        <button class="btn btn-secondary" on:click=move |_| perform_lock()>"LOCK & EXIT"</button>

                        {move || {
                            let msg = error_msg.get();
                            (!msg.is_empty()).then(|| view! { <p class="error">{msg}</p> })
                        }}
                    </div>
                })),

                AppView::AddBundle => Either::Right(Either::Right(view! {
                    <div class="view-add">
                        <h3>{move || if is_editing.get() { "EDIT BUNDLE" } else { "ADD NEW BUNDLE" }}</h3>
                        <div class="form-group">
                            <label>"BUNDLE NAME"</label>
                            <input
                                type="text"
                                placeholder="e.g. some::text::etc"
                                prop:disabled=move || is_editing.get()
                                prop:value=new_bundle_name
                                on:input=move |ev| set_new_bundle_name.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="form-group">
                            <label>{move || if is_editing.get() { "CONTENT" } else { "PASSWORD / CONTENT" }}</label>
                            <textarea
                                rows="5"
                                placeholder="Enter content or generate"
                                prop:value=new_bundle_pass
                                on:input=move |ev| set_new_bundle_pass.set(event_target_value(&ev))
                            ></textarea>
                            <button class="btn btn-ghost mt-1" on:click=generate_password>"GENERATE PASSWORD"</button>
                        </div>
                        <div style:display=move || if show_delete_confirm.get() { "none" } else { "block" }>
                            <button class="btn" on:click=move |_| perform_save_bundle()>"SAVE BUNDLE"</button>
                            {move || is_editing.get().then(|| view! {
                                <button class="btn btn-danger mt-1" on:click=move |_| set_show_delete_confirm.set(true)>"DELETE BUNDLE"</button>
                            })}
                            <button class="btn btn-ghost" on:click=move |_| set_view.set(AppView::Dashboard)>"CANCEL"</button>
                        </div>

                        {move || show_delete_confirm.get().then(|| view! {
                            <div class="confirm-box mt-1">
                                <p class="warning">"DELETE THIS BUNDLE?"</p>
                                <button class="btn btn-danger" on:click=move |_| perform_delete()>"CONFIRM DELETE"</button>
                                <button class="btn btn-ghost" on:click=move |_| set_show_delete_confirm.set(false)>"CANCEL"</button>
                            </div>
                        })}
                        
                        {move || {
                            let msg = error_msg.get();
                            (!msg.is_empty()).then(|| view! { <p class="error">{msg}</p> })
                        }}
                    </div>
                })),
            }}
        </div>
    }
}
