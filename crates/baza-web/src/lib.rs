use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use baza_core::{container, generate, BazaR};

#[derive(Clone, Debug)]
struct Line {
    content: String,
    is_user: bool,
}

#[component]
pub fn App() -> impl IntoView {
    let (history, set_history) = signal(vec![
        Line {
            content: "Welcome to Baza v2.9.0 (WASM)".to_string(),
            is_user: false,
        },
        Line {
            content: "Type 'help' for available commands.".to_string(),
            is_user: false,
        },
    ]);
    let (input, set_input) = signal(String::new());
    let (is_locked, set_is_locked) = signal(true);
    let input_ref = NodeRef::<leptos::html::Input>::new();

    let add_output = move |msg: &str| {
        set_history.update(|h| {
            h.push(Line {
                content: msg.to_string(),
                is_user: false,
            })
        });
    };

    let show_help = move || {
        add_output("Baza Commands:");
        add_output("  init [passphrase]          - Initialize vault");
        add_output("  unlock <passphrase>        - Unlock vault");
        add_output("  lock                       - Lock vault");
        add_output("  list                       - List all bundles");
        add_output("  bundle add <name>          - Create new bundle");
        add_output("  bundle generate <name>     - Generate bundle with random password");
        add_output("  bundle show <name>         - Show bundle contents");
        add_output("  bundle edit <name>         - Edit bundle");
        add_output("  bundle delete <name>       - Delete bundle");
        add_output("  bundle search <pattern>    - Search bundles");
        add_output("  bundle copy <name>         - Copy bundle to clipboard");
        add_output("  password generate [len]    - Generate random password");
        add_output("  clear                      - Clear terminal");
        add_output("  help                       - Show this help");
    };

    let handle_keydown = move |ev: KeyboardEvent| {
        if ev.key() == "Enter" {
            let cmd = input.get();
            if cmd.trim().is_empty() {
                return;
            }

            // Add user command to history
            set_history.update(|h| {
                h.push(Line {
                    content: cmd.clone(),
                    is_user: true,
                })
            });

            // Parse and process command
            let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
            if !parts.is_empty() {
                let response = match parts[0] {
                    "help" => {
                        show_help();
                        None
                    }
                    "clear" => {
                        set_history.set(vec![]);
                        None
                    }
                    "init" => {
                        let passphrase = parts.get(1).map(|s| s.to_string());
                        match process_init(passphrase) {
                            Ok(msg) => {
                                set_is_locked.set(false);
                                Some(msg)
                            }
                            Err(e) => Some(format!("Error: {}", e)),
                        }
                    }
                    "unlock" => {
                        if parts.len() < 2 {
                            Some("Error: passphrase required".to_string())
                        } else {
                            match process_unlock(parts[1].to_string()) {
                                Ok(msg) => {
                                    set_is_locked.set(false);
                                    Some(msg)
                                }
                                Err(e) => Some(format!("Error: {}", e)),
                            }
                        }
                    }
                    "lock" => {
                        match process_lock() {
                            Ok(msg) => {
                                set_is_locked.set(true);
                                Some(msg)
                            }
                            Err(e) => Some(format!("Error: {}", e)),
                        }
                    }
                    "list" => {
                        if is_locked.get() {
                            Some("Error: Vault is locked. Use 'unlock <password>'".to_string())
                        } else {
                            match process_list() {
                                Ok(msg) => Some(msg),
                                Err(e) => Some(format!("Error: {}", e)),
                            }
                        }
                    }
                    "bundle" => {
                        if is_locked.get() {
                            Some("Error: Vault is locked. Use 'unlock <password>'".to_string())
                        } else if parts.len() < 2 {
                            Some("Error: bundle command requires subcommand (add, generate, show, edit, delete, search, copy)".to_string())
                        } else {
                            match parts[1] {
                                "add" => {
                                    if parts.len() < 3 {
                                        Some("Error: bundle add requires name".to_string())
                                    } else {
                                        match process_bundle_add(parts[2].to_string()) {
                                            Ok(msg) => Some(msg),
                                            Err(e) => Some(format!("Error: {}", e)),
                                        }
                                    }
                                }
                                "generate" => {
                                    if parts.len() < 3 {
                                        Some("Error: bundle generate requires name".to_string())
                                    } else {
                                        match process_bundle_generate(parts[2].to_string()) {
                                            Ok(msg) => Some(msg),
                                            Err(e) => Some(format!("Error: {}", e)),
                                        }
                                    }
                                }
                                "show" => {
                                    if parts.len() < 3 {
                                        Some("Error: bundle show requires name".to_string())
                                    } else {
                                        match process_bundle_show(parts[2].to_string()) {
                                            Ok(msg) => Some(msg),
                                            Err(e) => Some(format!("Error: {}", e)),
                                        }
                                    }
                                }
                                "edit" => {
                                    if parts.len() < 3 {
                                        Some("Error: bundle edit requires name".to_string())
                                    } else {
                                        match process_bundle_edit(parts[2].to_string()) {
                                            Ok(msg) => Some(msg),
                                            Err(e) => Some(format!("Error: {}", e)),
                                        }
                                    }
                                }
                                "delete" => {
                                    if parts.len() < 3 {
                                        Some("Error: bundle delete requires name".to_string())
                                    } else {
                                        match process_bundle_delete(parts[2].to_string()) {
                                            Ok(msg) => Some(msg),
                                            Err(e) => Some(format!("Error: {}", e)),
                                        }
                                    }
                                }
                                "search" => {
                                    if parts.len() < 3 {
                                        Some("Error: bundle search requires pattern".to_string())
                                    } else {
                                        match process_bundle_search(parts[2].to_string()) {
                                            Ok(msg) => Some(msg),
                                            Err(e) => Some(format!("Error: {}", e)),
                                        }
                                    }
                                }
                                "copy" => {
                                    if parts.len() < 3 {
                                        Some("Error: bundle copy requires name".to_string())
                                    } else {
                                        match process_bundle_copy(parts[2].to_string()) {
                                            Ok(msg) => Some(msg),
                                            Err(e) => Some(format!("Error: {}", e)),
                                        }
                                    }
                                }
                                _ => Some("Error: Unknown bundle subcommand".to_string()),
                            }
                        }
                    }
                    "password" => {
                        if is_locked.get() {
                            Some("Error: Vault is locked. Use 'unlock <password>'".to_string())
                        } else if parts.len() < 2 {
                            Some("Error: password command requires subcommand (generate)".to_string())
                        } else {
                            match parts[1] {
                                "generate" => {
                                    let length = parts.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(24);
                                    match process_password_generate(length) {
                                        Ok(msg) => Some(msg),
                                        Err(e) => Some(format!("Error: {}", e)),
                                    }
                                }
                                _ => Some("Error: Unknown password subcommand".to_string()),
                            }
                        }
                    }
                    "version" => {
                        Some("Baza v2.9.0 (WASM)".to_string())
                    }
                    _ => Some("Command not found. Type 'help'.".to_string()),
                };

                if let Some(response) = response {
                    add_output(&response);
                }
            }

            set_input.set(String::new());
        }
    };

    // Auto-focus input on click anywhere
    let focus_input = move |_| {
        if let Some(input) = input_ref.get() {
            let _ = input.focus();
        }
    };

    view! {
        <div class="container" on:click=focus_input>
            <h1>"Baza Terminal"</h1>
            <div class={move || if is_locked.get() { "terminal-output locked" } else { "terminal-output" }}>
                <For
                    each=move || history.get()
                    key=|line| (line.content.clone(), line.is_user)
                    children=move |line| {
                        view! {
                            <div class="line">
                                <Show
                                    when=move || line.is_user
                                    fallback=|| view! { <span class="prefix">"  "</span> }
                                >
                                    <span class="prompt">"baza>"</span>
                                </Show>
                                <span class="content">{line.content}</span>
                            </div>
                        }
                    }
                />

                <div class="input-line">
                    <span class="prompt">"baza>"</span>
                    <input
                        node_ref=input_ref
                        type="text"
                        class="terminal-input"
                        prop:value=input
                        on:input=move |ev| set_input.set(event_target_value(&ev))
                        on:keydown=handle_keydown
                        autofocus
                        disabled=move || is_locked.get()
                    />
                </div>
            </div>
        </div>
    }
}

fn process_init(passphrase: Option<String>) -> Result<String, String> {
    let passphrase = passphrase.unwrap_or_else(|| uuid::Uuid::new_v4().hyphenated().to_string());
    baza_core::init(Some(passphrase.clone()))
        .map_err(|e| format!("{}", e))?;
    Ok(format!("Vault initialized! Passphrase: {}", passphrase))
}

fn process_unlock(passphrase: String) -> Result<String, String> {
    baza_core::unlock(Some(passphrase))
        .map_err(|e| format!("{}", e))?;
    Ok("Vault unlocked!".to_string())
}

fn process_lock() -> Result<String, String> {
    baza_core::lock()
        .map_err(|e| format!("{}", e))?;
    Ok("Vault locked!".to_string())
}

fn process_list() -> Result<String, String> {
    Ok("Listed all bundles (feature in progress)".to_string())
}

fn process_bundle_add(name: String) -> Result<String, String> {
    Ok(format!("Bundle '{}' created (feature in progress)", name))
}

fn process_bundle_generate(name: String) -> Result<String, String> {
    Ok(format!("Bundle '{}' generated with random password (feature in progress)", name))
}

fn process_bundle_show(name: String) -> Result<String, String> {
    Ok(format!("Bundle '{}' contents (feature in progress)", name))
}

fn process_bundle_edit(name: String) -> Result<String, String> {
    Ok(format!("Bundle '{}' edited (feature in progress)", name))
}

fn process_bundle_delete(name: String) -> Result<String, String> {
    Ok(format!("Bundle '{}' deleted (feature in progress)", name))
}

fn process_bundle_search(pattern: String) -> Result<String, String> {
    Ok(format!("Bundles matching '{}' (feature in progress)", pattern))
}

fn process_bundle_copy(name: String) -> Result<String, String> {
    Ok(format!("Bundle '{}' copied to clipboard (feature in progress)", name))
}

fn process_password_generate(length: usize) -> Result<String, String> {
    let password = generate(length, false, false, false)
        .map_err(|e| format!("{}", e))?;
    Ok(format!("Generated password: {}", password))
}
