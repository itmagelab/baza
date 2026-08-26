use baza_core::dump::{dump, restore, Algorithm};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct S {
    a: u32,
    b: String,
}

#[test]
fn roundtrip_none() {
    let s = S {
        a: 42,
        b: "hello".into(),
    };
    let dumped = match dump(&s, Algorithm::None) {
        Ok(d) => d,
        Err(e) => panic!("dump failed: {}", e),
    };
    let restored: S = match restore(&dumped) {
        Ok(r) => r,
        Err(e) => panic!("restore failed: {}", e),
    };
    assert_eq!(restored, s);
}

#[test]
fn roundtrip_lz4() {
    let s = S {
        a: 123,
        b: "some longer text to benefit from compression".into(),
    };
    let dumped = match dump(&s, Algorithm::Lz4) {
        Ok(d) => d,
        Err(e) => panic!("dump failed: {}", e),
    };
    let restored: S = match restore(&dumped) {
        Ok(r) => r,
        Err(e) => panic!("restore failed: {}", e),
    };
    assert_eq!(restored, s);
}
