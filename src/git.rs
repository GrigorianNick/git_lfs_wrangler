use std::process::Command;
use std::os::windows::process::CommandExt;
use std::sync::LazyLock;

use crate::lock::lock;
use crate::lock::lockstore::monothread_lockstore::MonothreadLockStore;
use crate::lock::lockstore::LockStore;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn get_user() -> String {
    let out = Command::new("cmd").args(["/C", "git config --get user.name"]).creation_flags(CREATE_NO_WINDOW).output();
    match out {
        Err(_) => "".to_string(),
        Ok(output) => {
            let mut s = String::from_utf8_lossy(&output.stdout).to_string();
            if s.ends_with('\n') {
                s.pop();
            }
            s.replace(" ", "_")
        }
    }
}

fn test_lock_string() -> String {
    ["I___", get_user().as_str()].join("")
}

pub fn is_lock_test(lock: &lock::LfsLock) -> bool {
    lock.file.starts_with("I___")
}

static LFS_USER: LazyLock<String> = LazyLock::new(|| {
    let store = MonothreadLockStore::new();
    match store.get_lock_file(&test_lock_string()) {
        Some(lock) => lock.owner.clone(),
        None => {
            match store.lock_file_fetch(&test_lock_string()) {
                None => String::from("UNKNOWN"),
                Some(new_lock) => new_lock.owner.clone()
            }
        }
    }
});

pub fn get_lfs_user() -> String {
    (&*LFS_USER).clone()
}

pub fn get_branch() -> String {
    let out = Command::new("cmd").args(["/C", "git branch --show-current"]).creation_flags(CREATE_NO_WINDOW).output();
    match out {
        Err(_) => "".to_string(),
        Ok(output) => {
            let mut s = String::from_utf8_lossy(&output.stdout).to_string();
            if s.ends_with('\n') {
                s.pop();
            }
            s
        }
    }
}