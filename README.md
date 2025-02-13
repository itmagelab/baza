# baza

![demo](contrib/Baza.gif)

## Installation

    cargo install baza

## Usage

Generate a new key for baza

    baza init

> [!WARNING]
> This is not an idempotent operation

### Re-init your baza

    baza init -p my_secret_pass_phrase
    baza --help

### Generate a new password by baza

    baza password generate 10
    baza password generate 30 --no-latters --no-symbols

### Create your baza bundles

    baza bundle create full::path::for::login
    baza bundle create work::depart::ldap::username
    baza bundle create site::google::username@gmail.com

### Delete your baza bundles

    baza bundle delete full::path::for::login

### Edit your bundle

    baza bundle search login
    baza bundle edit full::path::for::login

### Lock and Unlock your database (or bundles) with password phrase

    baza lock
    baza unlock

### Copy to clipboard

    baza bundle copy full::path::for::login
    baza --copy full::path::for::login

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

## Example libs usage

<https://habr.com/ru/companies/otus/articles/833714/>

## TODO

* Sync from a cloud providers
* TOTP
