use std::process::Command;
use std::os::windows::process::CommandExt;

use crate::lock::lock;
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

pub fn get_lfs_user(store: &dyn LockStore) -> String {
    println!("Checking for lfs user lock at: {}", &test_lock_string());
    match store.get_lock_file(&test_lock_string()) {
        Some(lock) => lock.owner.clone(),
        None => {
            println!("Does not exist!");
            match store.lock_file_fetch(&test_lock_string()) {
                None => {
                    println!("Failed to find lfs user lock: {}", &test_lock_string());
                    "".into()
                },
                Some(new_lock) => new_lock.owner.clone()
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