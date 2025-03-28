# baza

![RUST](https://github.com/itmagelab/baza/actions/workflows/rust.yaml/badge.svg)
![DOCKER](https://github.com/itmagelab/baza/actions/workflows/docker.yaml/badge.svg)

![demo](https://s697sas.storage.yandex.net/rdisk/a71dc8ecd15df180eff2883d6a7746506613baf9269c1d955e9dae9cb2851f2e/67e66215/TKGbzFHPUiRnpMvxkjcVTzg6SSbru5ud87MQEiOF2ng9vR7459uw0IyCdo9cHVO58LAxdQxcxr5cXD-pGM8NSg==?uid=0&filename=Baza.gif&disposition=inline&hash=&limit=0&content_type=image%2Fgif&owner_uid=0&fsize=998298&hid=ce66e3a721af9c798e3c1e9753c525f3&media_type=image&tknv=v2&etag=f3f0fb3a51d76aa3c88a1639ad1383b1&ts=631631e1cef40&s=29d6368812c0fc49a8e0bff14f6ad0160181f226191365e6cb164d0f1e646d19&pb=U2FsdGVkX1_M8BHS0aOOHmiHmylOwdDpED0ajs-fsPxvL2K6O2cuUDdCOB0Fa7ZQ5iY54OWSsoKadgXSV3IlsWxfF-qlehuIYzaXku0unkE)

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

![LOGO](https://cdn-ru.bitrix24.ru/b21763262/landing/189/1894d6c1c7ce75fed711df492921cfaf/3_1x.png)
