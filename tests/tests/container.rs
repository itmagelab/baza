use baza_core::container::{add, delete, read};
use baza_core::{unlock, Config, Password};
use baza_tests::{test_datadir, TEST_MUTEX};

fn create(str: &str) {
    let str = str.to_string();
    let password = Password::generate(255, false, false, false).as_str();
    match pollster::block_on(add(str, Some(password))) {
        Ok(_) => {}
        Err(e) => panic!("add failed: {}", e),
    }
}

fn read_test(str: &str) {
    let str = str.to_string();
    match pollster::block_on(read(str)) {
        Ok(_) => {}
        Err(e) => panic!("read failed: {}", e),
    }
}

fn delete_test(str: &str) {
    let str = str.to_string();
    match pollster::block_on(delete(str)) {
        Ok(_) => {}
        Err(e) => panic!("delete failed: {}", e),
    }
}

#[test]
fn it_works() {
    let _lock = TEST_MUTEX.lock().unwrap();
    let test_dir = std::path::PathBuf::from(test_datadir());
    let _ = std::fs::remove_dir_all(&test_dir);
    std::fs::create_dir_all(&test_dir).expect("Failed to create test dir");

    let config_path = test_dir.join("baza.toml");
    let mut config = Config::default();
    config.main.datadir = test_dir.to_string_lossy().to_string();
    let config_str = match toml::to_string(&config) {
        Ok(s) => s,
        Err(e) => panic!("toml serialize failed: {}", e),
    };
    if let Err(e) = std::fs::write(&config_path, config_str) {
        panic!("write config failed: {}", e);
    }
    if let Err(e) = Config::build(&config_path) {
        panic!("Config::build failed: {}", e);
    }

    let password = Password::generate(255, false, false, false).as_str();
    if let Err(e) = pollster::block_on(baza_core::init(Some(password.clone()))) {
        panic!("init failed: {}", e);
    }
    if let Err(e) = baza_core::utils::cleanup_tmp_folder() {
        panic!("cleanup failed: {}", e);
    }
    if let Err(e) = baza_core::lock() {
        panic!("lock failed: {}", e);
    }

    if let Err(e) = pollster::block_on(unlock(password.clone(), None)) {
        panic!("unlock failed: {}", e);
    }
    let bundles = vec![
        "test::my.test::login.ru",
        "test::my@test::login@ru",
        "test::my/test::login/ru",
        "test::my-test::login-ru",
        "test::my_test::login_ru",
    ];
    for name in bundles {
        create(name);
        read_test(name);
        delete_test(name);
    }
}

#[test]
fn test_create_from_str() {
    use baza_core::container::ContainerBuilder;

    let _lock = TEST_MUTEX.lock().unwrap();
    // Ensure Config is initialized
    let _ = Config::get();

    // Case 1: Multiple boxes and a bundle
    let builder = ContainerBuilder::new()
        .create_from_str("box1::box2::bundle1".to_string())
        .unwrap();
    let container = builder.build();
    assert_eq!(container.boxes.len(), 2);
    assert_eq!(&*container.boxes[0].borrow().name, "box1");
    assert_eq!(&*container.boxes[1].borrow().name, "box2");
    assert_eq!(container.bundles(), vec!["bundle1".to_string()]);

    // Case 2: Only bundle (no boxes) -> should fail
    let builder_res = ContainerBuilder::new().create_from_str("bundle_only".to_string());
    assert!(builder_res.is_err());

    // Case 3: Trim spaces from name but keep spaces inside delimiter-separated parts
    let builder = ContainerBuilder::new()
        .create_from_str("  spaced_box :: spaced_bundle  ".to_string())
        .unwrap();
    let container = builder.build();
    assert_eq!(container.boxes.len(), 1);
    assert_eq!(&*container.boxes[0].borrow().name, "spaced_box ");
    assert_eq!(container.bundles(), vec![" spaced_bundle".to_string()]);
}
