use baza_core::storage::{get_content, initialize, save_content, with_backend};
use baza_core::utils::as_hash;
use baza_core::{
    get_stored_salt, init, lock, migrate, set_session_key, unlock, SALT_KEY,
};
use baza_tests::{setup_test_env, TEST_MUTEX};

#[test]
fn test_migrate_legacy_sha256_to_argon2() {
    let _lock = TEST_MUTEX.lock().unwrap();
    setup_test_env();

    pollster::block_on(async {
        initialize().expect("Failed to initialize storage");

        // Simulate a legacy database: SHA-256 key derivation, no salt stored
        let passphrase = "legacy_passphrase";
        set_session_key(Some(as_hash(passphrase).to_vec())).unwrap();
        save_content("test::key".to_string(), "secret_value".to_string())
            .await
            .expect("Failed to store test value");

        // Sanity check: legacy value is readable with the SHA-256 key
        assert_eq!(get_content("test::key").await.unwrap(), "secret_value");

        // Migrate to Argon2
        migrate(passphrase.to_string())
            .await
            .expect("Migration failed");

        // Salt must be stored after migration
        assert!(get_stored_salt().await.is_some());

        // Value must still be readable with the new Argon2 key
        assert_eq!(get_content("test::key").await.unwrap(), "secret_value");

        // Second migration must be rejected
        assert!(migrate(passphrase.to_string()).await.is_err());

        // Wrong passphrase must fail during migration
        lock().unwrap();
        // Reset to legacy state for the wrong-passphrase check
        set_session_key(Some(as_hash("legacy_passphrase").to_vec())).unwrap();
        // Wipe salt to simulate legacy DB again
        with_backend(|backend| backend.remove(SALT_KEY))
            .await
            .expect("Failed to remove salt");
        assert!(migrate("wrong_passphrase".to_string()).await.is_err());
    });
}

#[test]
fn test_unlock_new_database_uses_salt() {
    let _lock = TEST_MUTEX.lock().unwrap();
    setup_test_env();

    pollster::block_on(async {
        init(Some("init_passphrase".to_string()))
            .await
            .expect("Failed to init database");

        assert!(get_stored_salt().await.is_some());

        // Round-trip: lock, unlock, read back
        save_content("test::key".to_string(), "secret_value".to_string())
            .await
            .expect("Failed to store test value");
        lock().unwrap();
        unlock("init_passphrase".to_string(), None)
            .await
            .expect("Failed to unlock with correct passphrase");
        assert_eq!(get_content("test::key").await.unwrap(), "secret_value");
    });
}
