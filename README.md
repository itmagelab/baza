# baza

![RUST](https://github.com/itmagelab/baza/actions/workflows/rust.yaml/badge.svg)
![DOCKER](https://github.com/itmagelab/baza/actions/workflows/docker.yaml/badge.svg)

![demo](contrib/Baza.gif)

## Description

This project is created as an alternative to password-store, but written in a low-level language with additional features

## Why should I use baza?

* Because it's very blazzing fast

## Installation

#### Docker ([ghcr.io](https://github.com/itmagelab/baza/pkgs/container/baza))

    docker run -ti -v "${HOME}/.baza:/usr/share/baza/.baza:rw" ghcr.io/itmagelab/baza:release-v2.6.1 baza --help

#### Cargo ([crates.io](https://crates.io/crates/baza))

> [!WARNING]
> Minimum Supported Rust Version: 1.83

    cargo install baza

#### Building

Baza compiles with Rust 1.83.0 (stable) or newer.

    git clone https://github.com/itmagelab/baza
    cd baza
    cargo build --release
    ./target/release/baza --version
    cp ./target/release/baza ~/.cargo/bin/

## Usage

Generate a new key for baza

    baza init

This command will generate a password phrase automatically, can be used for automations and CIs

> [!WARNING]
> !!! This is not an idempotent operation !!!
>
> When you create a new key, the old one is deleted without warning and the data cannot be recovered if you forget the password phrase

#### Re-init your baza

    baza init -p my_secret_pass_phrase
    baza --help

#### Generate a new password by baza

    baza password generate 10
    baza password generate 30 --no-latters --no-symbols

#### Create your baza bundles

    baza bundle create full::path::for::login
    baza bundle create work::depart::ldap::username
    baza bundle create site::google::username@gmail.com

#### Delete your baza bundles

    baza bundle delete full::path::for::login

#### Edit your bundle

    baza bundle search login
    baza bundle edit full::path::for::login

#### Lock and Unlock your database (or bundles) with password phrase

    baza lock
    baza unlock

#### Copy password to clipboard (first line from bundle)

    baza bundle copy full::path::for::login
    baza --copy full::path::for::login

#### Create bundle password from stdin

    echo '$ecRet' | baza --stdin full::path::for::login

## How to keep your keys safe

    gpg --list-keys
    echo "daec1759-f713-4cb2-bae6-5817b22c9c6c" | gpg --encrypt --armor --recipient root@itmage.ru > key.asc
    gpg --decrypt key.asc

Save the key in a safe place

## Create a GPG key

    gpg --gen-key
    gpg --export --armor baza > public_key.asc

## Generate VHS articles

    vhs < Baza.tape

## Migration from pass

    bash contrib/pass-to-baza.sh

## TODO

* Sync from a cloud providers
* TOTP

## Web Interface

Baza also provides a web interface for managing your passwords. The web interface is available at [https://0ae.ru](https://0ae.ru).

The database is stored locally in the user's browser and is not transmitted anywhere. You can create an unlimited number of bundles with passwords, and all of them can be unlocked with different passwords. The database can also be dumped to your cloud storage and then deployed in another location to continue working.

![LOGO](https://cdn-ru.bitrix24.ru/b21763262/landing/189/1894d6c1c7ce75fed711df492921cfaf/3_1x.png)
