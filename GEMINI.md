# Baza: Project Context and Coding Instructions

This file serves as the primary system-level context and reference guide for Gemini (the AI assistant) when working on the **Baza** project. Read this file at the start of any conversation or task to align on architectural design, code style, conventions, and workflow rules.

---

## 📖 Key Documentation References

Before starting any design or implementation, refer to the following local documents:
- **Development Conventions:** [doc/conventions.md](file:///Users/wilful/Git/baza/doc/conventions.md) *(Mandatory coding rules, error handling guidelines, and naming conventions)*
- **Development Workflow:** [doc/workflow.md](file:///Users/wilful/Git/baza/doc/workflow.md) *(Step-by-step process for planning, getting architectural approval, implementation, and logging progress)*
- **Task List & Backlog:** [doc/tasklist.md](file:///Users/wilful/Git/baza/doc/tasklist.md) *(Current active tasks, iterations, and progress logs)*
- **Product Vision:** [doc/vision.md](file:///Users/wilful/Git/baza/doc/vision.md) *(High-level overview, CLI examples, and configurations)*
- **Initial Idea:** [doc/idea.md](file:///Users/wilful/Git/baza/doc/idea.md) *(Project description, core features, and target audience)*

---

## 🛠️ Project Structure

Baza is a cargo workspace consisting of three primary crates:

1. **`crates/baza-core`**
   - **Role:** Core library containing business logic, encryption/decryption, serialization, and storage abstraction.
   - **Storage Abstraction:** Uses the `StorageBackend` trait, supporting **Redb** (native embedded key-value DB) and **WebStorage** (WASM-based Rexie/IndexedDB).
   - **Security:** All sensitive data is encrypted using AES-256-GCM. Sensitive memory is securely managed.
2. **`crates/baza`**
   - **Role:** Native CLI interface binary.
   - **Argument Parsing:** Uses the `argh` library.
   - **Commands:** supports `init`, `unlock`, `lock`, `bundle`, `password`, `list`, `version`, `dump`, `restore`, etc.
3. **`crates/baza-web`**
   - **Role:** Web UI application.
   - **Technologies:** Built using **Yew** and compiled to WebAssembly (WASM).

---

## 📐 Core Coding Rules & Conventions

To maintain codebase safety, security, and elegance, you must strictly follow these rules:

### 1. Error Handling (Crucial!)
- **NO `unwrap()` or `expect()`:** Do not use panic-triggering methods unless absolutely necessary (Clipart lint `#![deny(clippy::unwrap_used)]` is active).
- **Result Type:** Always return results using `BazaR<T>` (which is a type alias for `exn::Result<T, exn::Exn<error::Error>>`).
- **Contextual Context:** Use the `or_raise` operator (from `exn::ResultExt`) to add human-readable and context-rich errors when propagating results.
- **No `anyhow`:** Do not use `anyhow`. Use `exn::Result` and custom variants defined in `crates/baza-core/src/error.rs`.

*Example of proper error handling:*
```rust
use exn::ResultExt;

let data = std::fs::read("file.txt")
    .or_raise(|| crate::error::Error::Message("Failed to read critical file".into()))?;
```

### 2. Naming Conventions
- **Snake Case:** Use `snake_case` for filenames, modules, functions, and variables.
- **Pascal Case:** Use `PascalCase` for structs, enums, and traits.
- **No `mod.rs`:** Do not create `mod.rs` files for any new modules; use the modern module structure instead.
- **Action-Oriented Functions:** Function names should start with verbs describing action (e.g., `create_box`, `unlock_vault`).
- **Noun-Based Variables:** Variable names should be nouns representing their contents (e.g., `box_name`, `passphrase`).

### 3. Security Requirements
- **AES-256-GCM:** All data at rest must be encrypted.
- **Memory Safety:** Sensitive memory (e.g., passphrases, raw keys) must be cleared immediately after use.
- **Secure Logs:** Never log passwords, private keys, or any other sensitive customer data.

---

## 🔄 Mandatory Workflow (KISS & Approval Process)

Before writing any code or making workspace modifications:

1. **Check the Current Task:** Refer to [doc/tasklist.md](file:///Users/wilful/Git/baza/doc/tasklist.md) to identify the current active iteration and task.
2. **Analyze & Formulate Plan:** Analyze the files, dependencies, and requirements.
3. **Ask for Architectural Approval:** Propose your proposed design in the chat *before* you change files. Provide:
   - Architectural summary.
   - Example signatures of new structs, traits, or functions.
   - Multi-step implementation checklist.
   - **WAIT** for the user to reply and approve the design.
4. **Implement Step-by-Step:** Work on exactly one subtask at a time. Run tests and verify the code continuously.
5. **Commit Messages:** Commits must be written **strictly in English**.
6. **Update Progress:** Mark tasks as completed in [doc/tasklist.md](file:///Users/wilful/Git/baza/doc/tasklist.md) upon verification.

---

## 💻 Daily Development Commands

Use these cargo commands inside the workspace:

- **Build Workspace:** `cargo build --workspace`
- **Check Code Quality:** `cargo check --workspace`
- **Run Tests:** `cargo test --workspace`
- **Format Code:** `cargo fmt --workspace`
- **Clippy Linter:** `cargo clippy --workspace`
- **Debug Run CLI:** `RUST_LOG=debug cargo run --bin baza -- [arguments]`
