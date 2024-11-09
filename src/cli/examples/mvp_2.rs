/// container
/// box::box::box::bundle
///
/// ```bash
/// one::two::three::login
/// one::two::some
/// one
/// ```
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Default)]
pub struct Container {
    pub(crate) name: String,
    pub(crate) boxes: Vec<Rc<RefCell<r#Box>>>,
}

#[derive(Debug, Clone, Default)]
pub struct r#Box {
    pub(crate) name: String,
    pub(crate) bundle: Vec<Bundle>,
    parent: Option<Rc<RefCell<r#Box>>>,
}

#[derive(Debug, Clone, Default)]
pub struct Bundle {
    name: String,
}

fn main() {
    let bundle = Bundle {
        name: "login".to_string(),
    };

    let one = Box {
        name: "one".to_string(),
        bundle: vec![],
        parent: None,
    };
    let one = Rc::new(RefCell::new(one));

    let two = Box {
        name: "two".to_string(),
        bundle: vec![],
        parent: Some(Rc::clone(&one)),
    };
    let two = Rc::new(RefCell::new(two));

    let three = Box {
        name: "three".to_string(),
        bundle: vec![],
        parent: Some(Rc::clone(&two)),
    };
    let three = Rc::new(RefCell::new(three));

    // &Option<T> -> Option<&T>
    let borrowed = three.borrow();
    let Some(parent) = borrowed.parent.as_ref() else {
        todo!();
    };
    parent.clone().borrow_mut().bundle.push(bundle);
    // let rc = parent.unwrap();
    dbg!(two);
}
