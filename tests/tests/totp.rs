use baza_core::totp::{disable, enable, get_totp, is_enabled, verify_code};
use baza_tests::{setup_test_env, TEST_MUTEX};

#[test]
fn test_totp_flow() {
    let _lock = TEST_MUTEX.lock().unwrap();
    setup_test_env();

    pollster::block_on(async {
        baza_core::init(Some("test_passphrase".to_string()))
            .await
            .expect("Failed to init database");

        // 1. Check is_enabled initially (should be false)
        assert!(!is_enabled().await.expect("is_enabled failed"));

        // 2. Enable TOTP
        let (secret, url, _qr) = enable().await.expect("enable failed");
        assert!(!secret.is_empty());
        assert!(url.contains("secret="));

        // 3. Check is_enabled again (should be true)
        assert!(is_enabled().await.expect("is_enabled failed"));

        // 4. Verify code
        let totp = get_totp(&secret).expect("get_totp failed");
        let code = totp.generate_current().expect("generate_current failed");
        let valid = verify_code(&secret, &code).expect("verify_code failed");
        assert!(valid);

        // Verify invalid code
        let invalid = verify_code(&secret, "000000").expect("verify_code failed");
        assert!(!invalid);

        // 5. Disable TOTP
        disable().await.expect("disable failed");

        // 6. Check is_enabled (should be false)
        assert!(!is_enabled().await.expect("is_enabled failed"));
    });
}
