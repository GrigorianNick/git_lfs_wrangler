use std::process::Command;
use std::os::windows::process::CommandExt;

use crate::lock::lock;

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
            s
        }
    }
}

fn test_lock_string() -> String {
    ["I___", get_user().as_str()].join("")
}

pub fn is_lock_test(lock: &lock::LfsLock) -> bool {
    lock.file.starts_with("I___")
}

pub fn get_lfs_user(store: &lock::LockStore) -> String {
    match store.get_lock_file(&test_lock_string()) {
        Some(lock) => lock.owner.clone(),
        None => {
            match store.lock_file_fetch(&test_lock_string()) {
                None => "".into(),
                Some(new_lock) => new_lock.owner
            }
        }
    }
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