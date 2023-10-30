use std::{collections::HashMap, sync::OnceLock};

use tokio::sync::Mutex;

pub struct Locks(pub OnceLock<HashMap<String, Mutex<()>>>);

impl Locks {
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }
}

#[macro_export]
macro_rules! take_lock {
    ($locks: expr, $key: expr) => {
        unsafe {
            if $locks.0.get().is_none() {
                $locks.0.set(std::collections::HashMap::default()).unwrap();
            }
            $locks
                .0
                .get_mut()
                .unwrap()
                .entry($key)
                .or_insert(tokio::sync::Mutex::default())
                .lock()
                .await
        }
    };
}
