# baza

## Usage

Generate a new key for baza

    ./baza init

> [!WARNING]
> This is not an idempotent operation

### Re-init your baza

    ./baza init -u daec1759-f713-4cb2-bae6-5817b22c9c6c

### Generate a new password by baza

    ./baza password generate 10
    ./baza password generate 30 --no-latters --no-symbols

### Create your baza bundles

    ./baza bundle create full::path::for::login
    ./baza bundle create work::depart::ldap::username
    ./baza bundle create site::google::username@gmail.com

### Edit your bundle

    ./baza bundle search login
    ./baza bundle edit full::path::for::login

### Copy to clipboard

    ./baza bundle copy full::path::for::login

## How to keep your keys safe

    gpg --list-keys
    echo "daec1759-f713-4cb2-bae6-5817b22c9c6c" | gpg --encrypt --armor --recipient root@itmage.ru > key.asc
    gpg --decrypt key.asc

Save the key in a safe place

## Create a GPG key

    gpg --gen-key
    gpg --export --armor baza > public_key.asc

## Example libs usage

<https://habr.com/ru/companies/otus/articles/833714/>

## TODO

* Sync from a cloud providers
* TOTP
