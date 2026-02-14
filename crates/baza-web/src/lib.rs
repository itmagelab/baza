use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlElement, HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = encodeURIComponent)]
    fn uri_encode(s: &str) -> String;
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum AppView {
    Login,
    Dashboard,
    AddBundle,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct BundleInfo {
    name: String,
}

#[function_component(App)]
pub fn app() -> Html {
    let view = use_state(|| AppView::Login);
    let passphrase = use_state(String::new);
    let bundles = use_state(Vec::<BundleInfo>::new);
    let search_query = use_state(String::new);
    let error_msg = use_state(String::new);
    let show_init_confirm = use_state(|| false);
    let init_passphrase = use_state(|| None::<String>);

    // Add Bundle Form State
    let new_bundle_name = use_state(|| vec![String::new(), String::new(), String::new()]);
    let new_bundle_pass = use_state(String::new);
    let is_editing = use_state(|| false);
    let original_name = use_state(String::new);
    let show_delete_confirm = use_state(|| false);

    let load_bundles = {
        let bundles = bundles.clone();
        let error_msg = error_msg.clone();
        Callback::from(move |_| {
            let bundles = bundles.clone();
            let error_msg = error_msg.clone();
            spawn_local(async move {
                match baza_core::storage::list_all_keys().await {
                    Ok(keys) => {
                        let info = keys.into_iter().map(|name| BundleInfo { name }).collect();
                        bundles.set(info);
                    }
                    Err(e) => {
                        // If vault is locked, don't show error after successful restore
                        if !e.to_string().contains("Vault is locked") {
                            error_msg.set(format!("Load failed: {}", e));
                        }
                        // Clear bundles list when vault is locked
                        bundles.set(vec![]);
                    },
                }
            });
        })
    };

    let perform_unlock = {
        let passphrase = passphrase.clone();
        let set_view = view.clone();
        let error_msg = error_msg.clone();
        let set_init_passphrase = init_passphrase.clone();
        let load_bundles = load_bundles.clone();
        Callback::from(move |_| {
            let p = (*passphrase).clone();
            if p.is_empty() {
                error_msg.set("Passphrase cannot be empty".to_string());
                return;
            }
            let set_view = set_view.clone();
            let error_msg = error_msg.clone();
            let set_init_passphrase = set_init_passphrase.clone();
            let load_bundles = load_bundles.clone();

            match baza_core::unlock(Some(p)) {
                Ok(_) => {
                    set_init_passphrase.set(None);
                    set_view.set(AppView::Dashboard);
                    error_msg.set(String::new());
                    load_bundles.emit(());
                }
                Err(e) => error_msg.set(format!("Unlock failed: {}", e)),
            }
        })
    };

    let perform_init = {
        let passphrase = passphrase.clone();
        let set_view = view.clone();
        let error_msg = error_msg.clone();
        let set_init_passphrase = init_passphrase.clone();
        let set_show_init_confirm = show_init_confirm.clone();
        let set_bundles = bundles.clone();

        Callback::from(move |force: bool| {
            let passphrase = passphrase.clone();
            let set_view = set_view.clone();
            let error_msg = error_msg.clone();
            let set_init_passphrase = set_init_passphrase.clone();
            let set_show_init_confirm = set_show_init_confirm.clone();
            let set_bundles = set_bundles.clone();

            spawn_local(async move {
                if !force {
                    match baza_core::storage::is_initialized().await {
                        Ok(true) => {
                            set_show_init_confirm.set(true);
                            return;
                        }
                        Err(e) => {
                            error_msg.set(format!("Check failed: {}", e));
                            return;
                        }
                        Ok(false) => {}
                    }
                }

                let p = if passphrase.is_empty() {
                    None
                } else {
                    Some((*passphrase).clone())
                };

                match baza_core::init(p) {
                    Ok(key) => {
                        set_init_passphrase.set(Some(key));
                        set_show_init_confirm.set(false);
                        set_view.set(AppView::Dashboard);
                        error_msg.set(String::new());
                        set_bundles.set(vec![]);
                    }
                    Err(e) => error_msg.set(format!("Init failed: {}", e)),
                }
            });
        })
    };

    let perform_lock = {
        let set_view = view.clone();
        let set_passphrase = passphrase.clone();
        Callback::from(move |_| {
            let _ = baza_core::lock();
            set_view.set(AppView::Login);
            set_passphrase.set(String::new());
        })
    };

    let perform_save_bundle = {
        let name_state = new_bundle_name.clone();
        let pass_state = new_bundle_pass.clone();
        let old_name_state = original_name.clone();
        let is_editing_state = is_editing.clone();
        let set_view = view.clone();
        let error_msg = error_msg.clone();
        let load_bundles = load_bundles.clone();

        Callback::from(move |_| {
            let parts = (*name_state).clone();
            let name = parts
                .iter()
                .filter(|s| !s.is_empty())
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("::");
            let pass = (*pass_state).clone();
            let old_name = (*old_name_state).clone();
            let was_editing = *is_editing_state;
            let set_view = set_view.clone();
            let name_state = name_state.clone();
            let pass_state = pass_state.clone();
            let old_name_state = old_name_state.clone();
            let is_editing_state = is_editing_state.clone();
            let error_msg = error_msg.clone();
            let load_bundles = load_bundles.clone();

            if name.is_empty() || pass.is_empty() {
                error_msg.set("Name and content are required".to_string());
                return;
            }

            spawn_local(async move {
                match baza_core::container::add(name.clone(), Some(pass)).await {
                    Ok(_) => {
                        if was_editing && name != old_name {
                            let _ = baza_core::storage::delete_by_name(old_name).await;
                        }
                        set_view.set(AppView::Dashboard);
                        name_state.set(vec![String::new(), String::new(), String::new()]);
                        pass_state.set(String::new());
                        old_name_state.set(String::new());
                        is_editing_state.set(false);
                        error_msg.set(String::new());
                        load_bundles.emit(());
                    }
                    Err(e) => error_msg.set(format!("Save failed: {}", e)),
                }
            });
        })
    };

    let perform_edit = {
        let set_name = new_bundle_name.clone();
        let set_orig_name = original_name.clone();
        let set_pass = new_bundle_pass.clone();
        let set_is_editing = is_editing.clone();
        let set_show_delete_confirm = show_delete_confirm.clone();
        let set_view = view.clone();
        let error_msg = error_msg.clone();

        Callback::from(move |name: String| {
            let name_clone = name.clone();
            let set_name = set_name.clone();
            let set_orig_name = set_orig_name.clone();
            let set_pass = set_pass.clone();
            let set_is_editing = set_is_editing.clone();
            let set_show_delete_confirm = set_show_delete_confirm.clone();
            let set_view = set_view.clone();
            let error_msg = error_msg.clone();

            spawn_local(async move {
                match baza_core::storage::get_content(name_clone.clone()).await {
                    Ok(content) => {
                        // split existing name into parts and ensure at least 3 input fields
                        let mut parts: Vec<String> =
                            name_clone.split("::").map(|s| s.to_string()).collect();
                        while parts.len() < 3 {
                            parts.push(String::new());
                        }
                        set_name.set(parts);
                        set_orig_name.set(name_clone);
                        set_pass.set(content);
                        set_is_editing.set(true);
                        set_show_delete_confirm.set(false);
                        set_view.set(AppView::AddBundle);
                    }
                    Err(e) => error_msg.set(format!("Load failed: {}", e)),
                }
            });
        })
    };

    let perform_delete = {
        let orig_name_state = original_name.clone();
        let set_view = view.clone();
        let set_pass = new_bundle_pass.clone();
        let load_bundles = load_bundles.clone();
        let error_msg = error_msg.clone();
        let set_show_delete_confirm = show_delete_confirm.clone();

        Callback::from(move |_| {
            let name = (*orig_name_state).clone();
            let set_view = set_view.clone();
            let orig_name_state = orig_name_state.clone();
            let set_pass = set_pass.clone();
            let load_bundles = load_bundles.clone();
            let error_msg = error_msg.clone();
            let set_show_delete_confirm = set_show_delete_confirm.clone();

            spawn_local(async move {
                match baza_core::storage::delete_by_name(name).await {
                    Ok(_) => {
                        set_show_delete_confirm.set(false);
                        set_view.set(AppView::Dashboard);
                        orig_name_state.set(String::new());
                        set_pass.set(String::new());
                        load_bundles.emit(());
                    }
                    Err(e) => error_msg.set(format!("Delete failed: {}", e)),
                }
            });
        })
    };

    let perform_dump = {
        let error_msg = error_msg.clone();
        Callback::from(move |_| {
            let error_msg = error_msg.clone();
            spawn_local(async move {
                match baza_core::storage::dump().await {
                    Ok(data) => {
                        // Serialize+compress into custom .baza binary format
                        match baza_core::dump::dump(&data, baza_core::dump::Algorithm::Lz4) {
                            Ok(bytes) => {
                                if let Some(window) = web_sys::window() {
                                    if let Some(document) = window.document() {
                                        // Create a blob from bytes and download as .baza
                                        let u8 = js_sys::Uint8Array::from(bytes.as_slice());
                                        let arr = js_sys::Array::new();
                                        arr.push(&u8);
                                        if let Ok(blob) =
                                            web_sys::Blob::new_with_u8_array_sequence(&arr)
                                        {
                                            if let Ok(url) =
                                                web_sys::Url::create_object_url_with_blob(&blob)
                                            {
                                                if let Ok(el) = document.create_element("a") {
                                                    let a =
                                                        el.unchecked_into::<web_sys::HtmlElement>();
                                                    let timestamp = js_sys::Date::now();
                                                    let filename =
                                                        format!("baza_dump_{}.baza", timestamp);
                                                    let _ = a.set_attribute("href", &url);
                                                    let _ = a.set_attribute("download", &filename);
                                                    if let Some(body) = document.body() {
                                                        let _ = body.append_child(&a);
                                                        a.click();
                                                        let _ = body.remove_child(&a);
                                                    } else {
                                                        error_msg.set(
                                                            "Unable to access document body"
                                                                .to_string(),
                                                        );
                                                    }
                                                    // Revoke the object URL
                                                    let _ = web_sys::Url::revoke_object_url(&url);
                                                    error_msg.set("DATABASE DUMPED".to_string());
                                                    let error_msg = error_msg.clone();
                                                    spawn_local(async move {
                                                        gloo_timers::future::TimeoutFuture::new(
                                                            2000,
                                                        )
                                                        .await;
                                                        error_msg.set(String::new());
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => error_msg.set(format!("Dump failed: {}", e)),
                        }
                    }
                    Err(e) => error_msg.set(format!("Dump failed: {}", e)),
                }
            });
        })
    };

    let perform_restore = {
        let error_msg = error_msg.clone();
        let load_bundles = load_bundles.clone();
        Callback::from(move |e: Event| {
            let target_opt = e.target_dyn_into::<HtmlInputElement>();
            let target = match target_opt {
                Some(t) => t,
                None => {
                    error_msg.set("Invalid file input event".to_string());
                    return;
                }
            };
            if let Some(files) = target.files() {
                if let Some(file) = files.get(0) {
                    let error_msg = error_msg.clone();
                    let load_bundles = load_bundles.clone();
                    spawn_local(async move {
                        // Read file as ArrayBuffer for binary .baza format
                        let promise = file.array_buffer();
                        match wasm_bindgen_futures::JsFuture::from(promise).await {
                            Ok(buf_js) => {
                                let uint8 = js_sys::Uint8Array::new(&buf_js);
                                let bytes = uint8.to_vec();
                                match baza_core::dump::restore::<Vec<(String, Vec<u8>)>>(&bytes) {
                                    Ok(data) => match baza_core::storage::restore_unlocked(data).await {
                                        Ok(_) => {
                                            error_msg.set("RESTORE SUCCESSFUL".to_string());
                                            load_bundles.emit(());
                                            let error_msg = error_msg.clone();
                                            spawn_local(async move {
                                                gloo_timers::future::TimeoutFuture::new(2000).await;
                                                error_msg.set(String::new());
                                            });
                                        }
                                        Err(e) => error_msg.set(format!("Restore failed: {}", e)),
                                    },
                                    Err(e) => error_msg.set(format!("Parse failed: {}", e)),
                                }
                            }
                            Err(e) => error_msg.set(format!("Read failed: {:?}", e)),
                        }
                    });
                }
            }
        })
    };

    let perform_copy_first_line = {
        let error_msg = error_msg.clone();
        Callback::from(move |name: String| {
            let error_msg = error_msg.clone();
            spawn_local(async move {
                match baza_core::storage::get_content(name).await {
                    Ok(content) => {
                        let first_line = content.lines().next().unwrap_or("").trim().to_string();
                        let mut copied = false;
                        if let Some(window) = web_sys::window() {
                            let is_secure = window.is_secure_context();
                            if is_secure {
                                let nav = window.navigator();
                                let promise = nav.clipboard().write_text(&first_line);
                                if wasm_bindgen_futures::JsFuture::from(promise).await.is_ok() {
                                    copied = true;
                                }
                            }
                            if !copied {
                                if let Some(document) = window.document() {
                                    if let Ok(el) = document.create_element("textarea") {
                                        let textarea =
                                            el.unchecked_into::<web_sys::HtmlTextAreaElement>();
                                        textarea.set_value(&first_line);
                                        let style = textarea
                                            .unchecked_ref::<web_sys::HtmlElement>()
                                            .style();
                                        let _ = style.set_property("position", "fixed");
                                        let _ = style.set_property("left", "-9999px");
                                        let _ = style.set_property("top", "0");
                                        if let Some(body) = document.body() {
                                            let _ = body.append_child(&textarea);
                                            textarea.focus().ok();
                                            textarea.select();
                                            let html_doc =
                                                document.unchecked_into::<web_sys::HtmlDocument>();
                                            if html_doc.exec_command("copy").unwrap_or(false) {
                                                copied = true;
                                            }
                                            let _ = body.remove_child(&textarea);
                                        }
                                    }
                                }
                            }
                            if !copied {
                                let msg = if is_secure {
                                    "Automatic copy failed. Please copy manually from the field below:"
                                } else {
                                    "Insecure connection (HTTP). Automatic copy disabled. Use the field below:"
                                };
                                let _ = window.prompt_with_message_and_default(msg, &first_line);
                                error_msg.set("USE PROMPT TO COPY".to_string());
                                let error_msg = error_msg.clone();
                                spawn_local(async move {
                                    gloo_timers::future::TimeoutFuture::new(3000).await;
                                    error_msg.set(String::new());
                                });
                                return;
                            }
                        }

                        if copied {
                            error_msg.set("COPIED TO CLIPBOARD".to_string());
                            let error_msg = error_msg.clone();
                            spawn_local(async move {
                                gloo_timers::future::TimeoutFuture::new(2000).await;
                                error_msg.set(String::new());
                            });
                        } else {
                            error_msg.set("COPY FAILED".to_string());
                        }
                    }
                    Err(e) => error_msg.set(format!("Copy failed: {}", e)),
                }
            });
        })
    };

    let generate_password = {
        let set_pass = new_bundle_pass.clone();
        Callback::from(move |_| {
            if let Ok(p) = baza_core::generate(24, false, false, false) {
                set_pass.set(p);
            }
        })
    };

    let filtered_bundles = {
        let query = (*search_query).to_lowercase();
        bundles
            .iter()
            .filter(|b| b.name.to_lowercase().contains(&query))
            .cloned()
            .collect::<Vec<_>>()
    };

    let on_passphrase_input = {
        let set_passphrase = passphrase.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            set_passphrase.set(input.value());
        })
    };

    let on_search_input = {
        let set_search = search_query.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            set_search.set(input.value());
        })
    };

    // Bundle name parts input handler factory
    let on_bundle_part_input = {
        let parts_state = new_bundle_name.clone();
        Callback::from(move |(idx, e): (usize, InputEvent)| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut parts = (*parts_state).clone();
            if idx >= parts.len() {
                return;
            }
            parts[idx] = input.value();

            // If this is the last part and it's now non-empty, append a new empty part
            if idx + 1 == parts.len() && !parts[idx].trim().is_empty() {
                parts.push(String::new());
            }

            parts_state.set(parts);
        })
    };

    // Keydown handler: on Enter/Tab, if current part non-empty, append new part and focus it
    let on_bundle_part_keydown = {
        let parts_state = new_bundle_name.clone();
        Callback::from(move |(idx, e): (usize, KeyboardEvent)| {
            let key = e.key();
            if key != "Enter" && key != "Tab" {
                return;
            }
            e.prevent_default();
            let mut parts = (*parts_state).clone();
            if idx >= parts.len() {
                return;
            }
            if parts[idx].trim().is_empty() {
                return;
            }
            let next_idx = idx + 1;
            if next_idx >= parts.len() {
                parts.push(String::new());
                parts_state.set(parts.clone());
            }

            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    let id = format!("bundle-part-{}", next_idx);
                    if let Some(el) = document.get_element_by_id(&id) {
                        let _ = el.unchecked_into::<HtmlElement>().focus();
                    }
                }
            }
        })
    };

    let on_bundle_pass_input = {
        let set_pass = new_bundle_pass.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            set_pass.set(input.value());
        })
    };

    let on_passphrase_keydown = {
        let perform_unlock = perform_unlock.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                perform_unlock.emit(());
            }
        })
    };

    let parts_html = {
        let cb = on_bundle_part_input.clone();
        let kd = on_bundle_part_keydown.clone();
        (*new_bundle_name)
            .iter()
            .enumerate()
            .map(move |(i, part)| {
                let value = part.clone();
                let cb = cb.clone();
                let kd = kd.clone();
                let id = format!("bundle-part-{}", i);
                html! {
                    <input
                        key={i.to_string()}
                        id={id}
                        type="text"
                        placeholder={
                            if i == 0 {
                                "scope (e.g. github, token)".to_string()
                            } else if i == 1 {
                                "bundle (e.g. mylogin, my@email.com)".to_string()
                            } else if part.is_empty() {
                                "press Enter/Tab to advance".to_string()
                            } else {
                                "bundle part".to_string()
                            }
                        }
                        value={value}
                        oninput={Callback::from(move |e: InputEvent| cb.emit((i, e)))}
                        onkeydown={Callback::from(move |e: KeyboardEvent| kd.emit((i, e)))}
                    />
                }
            })
            .collect::<Html>()
    };

    html! {
        <div class="container">
            <h1>{"BAZA"}</h1>

            {
                match *view {
                    AppView::Login => html! {
                        <div class="view-login">
                            if *show_init_confirm {
                                <div class="confirm-box">
                                    <p class="warning">{"VAULT EXISTS! OVERWRITE?"}</p>
                                    <button class="btn btn-danger" onclick={
                                        let perform_init = perform_init.clone();
                                        move |_| perform_init.emit(true)
                                    }>{"CONTINUE"}</button>
                                    <button class="btn btn-ghost" onclick={
                                        let set_show_init_confirm = show_init_confirm.clone();
                                        move |_| set_show_init_confirm.set(false)
                                    }>{"CANCEL"}</button>
                                </div>
                            }

                            <div style={if *show_init_confirm { "display: none" } else { "display: block" }}>
                                <div class="form-group">
                                    <label>{"PASSPHRASE"}</label>
                                    <input
                                        type="password"
                                        placeholder="Enter your passphrase"
                                        value={(*passphrase).clone()}
                                        oninput={on_passphrase_input}
                                        onkeydown={on_passphrase_keydown}
                                    />
                                </div>
                                <button class="btn" onclick={
                                    let perform_unlock = perform_unlock.clone();
                                    move |_| perform_unlock.emit(())
                                }>{"UNLOCK"}</button>
                                <button class="btn btn-ghost" onclick={
                                    let perform_init = perform_init.clone();
                                    move |_| perform_init.emit(false)
                                }>{"INITIALIZE NEW VAULT"}</button>

                                if !error_msg.is_empty() {
                                    <p class="error">{(*error_msg).clone()}</p>
                                }

                                <div class="backup-actions mt-1">
                                    <label class="btn btn-ghost" style="text-align: center; display: block;">
                                        {"RESTORE DATABASE (.baza)"}
                                        <input
                                            type="file"
                                            accept=".baza"
                                            style="display: none"
                                            onchange={perform_restore}
                                        />
                                    </label>
                                </div>
                            </div>
                        </div>
                    },
                    AppView::Dashboard => html! {
                        <div class="view-dashboard">
                            if let Some(p) = (*init_passphrase).as_ref() {
                                <div class="passphrase-banner">
                                    <p>{"YOUR NEW PASSPHRASE:"}</p>
                                    <code>{p}</code>
                                    <p class="small">{"SAVE IT NOW. IT WILL NOT BE SHOWN AGAIN."}</p>
                                </div>
                            }

                            <div class="search-group">
                                <input
                                    type="text"
                                    placeholder="Search bundles..."
                                    value={(*search_query).clone()}
                                    oninput={on_search_input}
                                />
                                if !search_query.is_empty() {
                                    <button class="search-clear" onclick={
                                        let set_search_query = search_query.clone();
                                        move |_| set_search_query.set(String::new())
                                    }>{"×"}</button>
                                }
                            </div>

                            <ul class="bundle-list">
                                {
                                    for filtered_bundles.iter().map(|b| {
                                        let name = b.name.clone();
                                        let name_for_edit = name.clone();
                                        let name_for_copy = name.clone();
                                        let perform_edit = perform_edit.clone();
                                        let perform_copy = perform_copy_first_line.clone();
                                        html! {
                                            <li class="bundle-item" onclick={move |_| perform_copy.emit(name_for_copy.clone())}>
                                                <span class="bundle-name">{&name}</span>
                                                <div class="bundle-actions">
                                                    <button class="action-btn" title="Edit" onclick={move |e: MouseEvent| {
                                                        e.stop_propagation();
                                                        perform_edit.emit(name_for_edit.clone());
                                                    }}>{"✏️"}</button>
                                                </div>
                                            </li>
                                        }
                                    })
                                }
                            </ul>

                            <button class="btn" onclick={
                                let set_is_editing = is_editing.clone();
                                let set_show_delete_confirm = show_delete_confirm.clone();
                                let set_new_bundle_name = new_bundle_name.clone();
                                let set_new_bundle_pass = new_bundle_pass.clone();
                                let set_view = view.clone();
                                move |_| {
                                    set_is_editing.set(false);
                                    set_show_delete_confirm.set(false);
                                    set_new_bundle_name.set(vec![String::new(), String::new(), String::new()]);
                                    set_new_bundle_pass.set(String::new());
                                    set_view.set(AppView::AddBundle);
                                }
                            }>{"ADD NEW BUNDLE"}</button>



                            <div class="backup-actions mt-1">
                                <button class="btn btn-secondary" onclick={move |_| perform_dump.emit(())}>{"DUMP DATABASE"}</button>
                                <button class="btn btn-secondary ml-1" onclick={move |_| perform_lock.emit(())}>{"LOCK & EXIT"}</button>
                            </div>

                            if !error_msg.is_empty() {
                                <p class="error">{(*error_msg).clone()}</p>
                            }
                        </div>
                    },
                    AppView::AddBundle => html! {
                         <div class="view-add">
                            <h3>{if *is_editing { "EDIT BUNDLE" } else { "ADD NEW BUNDLE" }}</h3>
                            <div class="form-group">
                                <label>{"BUNDLE NAME"}</label>
                                { parts_html.clone() }
                            </div>
                            <div class="form-group">
                                <label>{if *is_editing { "CONTENT" } else { "PASSWORD / CONTENT" }}</label>
                                <textarea
                                    rows="5"
                                    placeholder="Enter content or generate"
                                    value={(*new_bundle_pass).clone()}
                                    oninput={on_bundle_pass_input}
                                ></textarea>
                                <button class="btn btn-ghost mt-1" onclick={move |_| generate_password.emit(())}>{"GENERATE PASSWORD"}</button>
                            </div>
                            <div style={if *show_delete_confirm { "display: none" } else { "display: block" }}>
                                <button class="btn" onclick={move |_| perform_save_bundle.emit(())}>{"SAVE BUNDLE"}</button>
                                if *is_editing {
                                    <button class="btn btn-danger mt-1" onclick={
                                        let set_show_delete_confirm = show_delete_confirm.clone();
                                        move |_| set_show_delete_confirm.set(true)
                                    }>{"DELETE BUNDLE"}</button>
                                }
                                <button class="btn btn-ghost" onclick={
                                    let set_view = view.clone();
                                    move |_| set_view.set(AppView::Dashboard)
                                }>{"CANCEL"}</button>
                            </div>

                            if *show_delete_confirm {
                                <div class="confirm-box mt-1">
                                    <p class="warning">{"DELETE THIS BUNDLE?"}</p>
                                    <button class="btn btn-danger" onclick={move |_| perform_delete.emit(())}>{"CONFIRM DELETE"}</button>
                                    <button class="btn btn-ghost" onclick={
                                        let set_show_delete_confirm = show_delete_confirm.clone();
                                        move |_| set_show_delete_confirm.set(false)
                                    }>{"CANCEL"}</button>
                                </div>
                            }

                            if !error_msg.is_empty() {
                                <p class="error">{(*error_msg).clone()}</p>
                            }
                        </div>
                    }
                }
            }
        </div>
    }
}
