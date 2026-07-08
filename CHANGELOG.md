# Changelog

All notable changes to this project will be documented in this file.

## [3.2.0] - 2026-07-08

### 🚀 Features

- *(cli)* Add unlock and lock subcommands to manage BAZA_PASSPHRASE environment variable
- *(cli)* Bypass vault unlock for password generate command
- Gate S3 push/pull sync support behind cargo feature to maintain small default binary size
- Implement baza push and pull commands for S3 sync
- Render TOTP QR code by default, adding --no-qr flag

### 🚜 Refactor

- Introduce metadata-driven pre-flight TOTP check and single-pass unlock
- Remove redundant restore_unlocked function

## [3.1.2] - 2026-06-26

### 🐛 Bug Fixes

- *(web)* Reset totp state when locking database or deleting database
- *(wasm)* Avoid unsupported system time panic on wasm32 targets during TOTP check

## [3.1.1] - 2026-06-26

### 🐛 Bug Fixes

- *(wasm)* Enable wasm_js feature for getrandom v0.3 on wasm32 target

## [3.1.0] - 2026-06-26

### 🚀 Features

- Support BAZA_CONFIG environment variable and add baza.dev.toml
- Generate and store random UUID for TOTP identity instead of hardcoded email
- Add TOTP authorization support to core, CLI, and Web UI

### 🚜 Refactor

- Update Baza.tape and Baza.gif for TOTP and clean database setup

### 📚 Documentation

- Add TOTP authentication instructions and usage examples to README

## [3.0.1] - 2026-06-25

### 🐛 Bug Fixes

- Provide delete_database stub in baza-core for non-WASM platforms

### 🚜 Refactor

- Drop database when initialize

### 📚 Documentation

- Add GEMINI.md project context file

## [3.0.0] - 2026-02-17

### 🚀 Features

- Added new button delete all
- Added dump/restore for cli
- Added baza format file and cleanup from unwraps
- Replaced bundle name placeholder and reaction
- Move dump/restore
- Added new compose file
- Use root image
- Added manual buileder
- Added web version on WASM
- *(web)* Redesign UI to modern minimalist style and refactor logic to state-based views
- Add `list` and `version` commands, and refactor CLI argument parsing to map top-level flags to subcommands.
- Switch to redb
- Try again use gix

### 🐛 Bug Fixes

- Interface errors
- Generation types
- Fixed animation and vault lock
- Uotput by cli
- Rename entrypoint script
- Homedir path fixed
- Remove trash from macos
- Remove none command
- Dependabot

### 🚜 Refactor

- Cleanup code
- Move buttons
- Using fmt
- WASM #1
- Minified binary size
- Remove git storage support
- Migrate CLI argument parsing from `clap` to `argh` and replace `regex` with `regex-lite`.
- Remove unused modules
- Rename `Storage` trait to `StorageBackend`, add a `sync` method, and centralize backend selection with a new `with_backend` helper function.
- Rename `Storage` trait to `StorageBackend`, add a `sync` method, and centralize backend selection with a new `with_backend` helper function.
- Update tasklist
- Cleanup unused tests
- Move a code to crates

### 📚 Documentation

- Updated status badge
- Updatet the documentation
- Update project conventions documentation.

### 🧪 Testing

- Move to Yew
- Debug for production running

## [2.9.0] - 2025-04-20

### 🚀 Features

- Added action for a tag
- Fixed a bug with too many open files
- Reename create to add
- Added auto password generation

### 🐛 Bug Fixes

- Cleanup
- Cleanup
- Extended tests

## [2.8.0] - 2025-03-28

### 🚀 Features

- Replace defaul search re
- Move from impl #3
- Move from impl #2
- Move from impl
- Ext as const
- Rename error
- Added types for box, bundle

### 🐛 Bug Fixes

- Bug with extention
- Cleanup Ctx

## [2.7.0] - 2025-03-28

### 🚀 Features

- Reword all boxes
- Added gix storage
- Storage using data paths
- Move git as trait storage
- Added sync wrapper
- Rework logging

### 🐛 Bug Fixes

- Use cleaner once time
- Make tmp when initialize a new bundle
- Rename methods

### 📚 Documentation

- Update animation
- Update animation
- Added new usage
- Rename shortcuts
- Added shortcut for show
- Added shortcut for copy
- Writting reame
- Update animation

## [2.6.1] - 2025-02-28

### 🐛 Bug Fixes

- Added readme for a crate
- Hotfix depends

## [2.6.0] - 2025-02-28

### 🚀 Features

- Added extention for files
- Reworked save fn for containers
- Rename for upload command

### 🐛 Bug Fixes

- Now can use names with dots
- For delete and edit fn
- For delete and edit fn
- Search separator
- Rename pipeline name

### 📚 Documentation

- Added script for migration
- Fixes packages list

## [2.5.0] - 2025-02-27

### 🚀 Features

- Use regex for searching

### 🐛 Bug Fixes

- Fixed help messages

### 📚 Documentation

- Added budges
- Added new feature to docs
- Added new docker release

## [2.4.0] - 2025-02-27

### 🚀 Features

- Use stdin for bundle creation

### 🐛 Bug Fixes

- No case sensitive
- Read all input for stdin
- Ignore if tmp cant remove

### 📚 Documentation

- Added discription for migration from pass

## [2.3.0] - 2025-02-21

### 🚀 Features

- Train fixes again

## [2.2.0] - 2025-02-18

### 🚀 Features

- List all containers command
- Simplified work with box vectors
- Added show command

## [2.1.0] - 2025-02-15

### 🚀 Features

- Added load command
- Added docker prebuild images
- Copy only first line to clipboard
- Rename config function
- Use info debug level
- Added description and new opts

### 🐛 Bug Fixes

- Fixing initializing git repo

### 📚 Documentation

- Added msrv
- Added link to crate package
- Update readme information
- Cleanup readme docs
- Added installation section

## [2.0.0] - 2025-02-13

### 🚀 Features

- [**breaking**] Rename baza crate for bunary
- Lock/unlock command was added
- Fn for show messages
- Arc for configuration
- Added animation
- Use in folder config
- Added first tests

### 🐛 Bug Fixes

- Fixed cargo configuration
- Added trim for asked passphrase
- GitHub actions fixes
- Cleanup folder
- Cleanup folder
- Cleanup from storage module

### 📚 Documentation

- Updated documentation and added new feature
- Update demo file
- Clarified the error description
- Added short description for commands
- Added more description

## [1.0.0] - 2025-02-09

### 🚀 Features

- Added default help message
- Use git2 error trait
- Added delete handlers
- Reworked git module
- Short arg for copy
- Reworked the main name fn
- Separate data folder for keys
- [**breaking**] Init for git usage
- New feature for clipboard
- Release for this AES
- Added first example enc/decr
- Try to use gpg
- Search added for bundle
- Added display trait and list handler
- Added edit command
- [**breaking**] Create a new api for creating bundles
- Added first changelog
- Added cliff

### 🐛 Bug Fixes

- Remove storage from cli
- Wrong commit name was changed
- Changed commit default
- Refact for a bundle methods
- Rename from_string

### 📚 Documentation

- Skipp all unlabeled commits
- First version of help info
- Added clipboard command
- Fix a docs
- Added base ussage for baza
- Added warnings

## [0.1.0] - 2024-12-14

<!-- generated by git-cliff -->
