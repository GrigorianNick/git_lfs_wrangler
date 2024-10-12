use crate::lock::LfsLock;
use crate::lock::tag::*;

use std::collections::HashMap;
use std::process::Command;
use std::os::windows::process::CommandExt;

use super::LockStore;

const CREATE_NO_WINDOW: u32 = 0x08000000;

fn normalize_path(p: &String) -> String {
    let s = p.replace("\\", "/");
    match s.strip_prefix("./") {
        None => s,
        Some(stripped) => stripped.to_string(),
    }
}

pub struct MonothreadLockStore {
    locks: Vec<LfsLock>,
}

impl Default for MonothreadLockStore {
    fn default() -> Self {
        MonothreadLockStore {
            locks: vec![],
        }
    }
}

impl MonothreadLockStore {

    pub fn new() -> Box<Self> {
        Box::new(MonothreadLockStore::default())
    }
}

impl LockStore for MonothreadLockStore {

    // Fetches raw locks
    fn get_raw_locks(&self) -> Vec<LfsLock> {
        let out = Command::new("cmd").args(["/C", "git lfs locks"]).creation_flags(CREATE_NO_WINDOW).output().expect("Failed to execute process");
        let out = String::from_utf8_lossy(&out.stdout).to_string();
        let lines: Vec<&str> = out.split("\n").filter(|&s| !s.is_empty()).collect();
        let locks: Vec<LfsLock> = lines.iter().map(|&l| LfsLock::from_line(l.to_string()).unwrap()).collect();
        locks
    }

    fn lock_file(&self, p: &String) -> bool {
        let out = Command::new("cmd").args(["/C".into(), ["git lfs lock ", p.as_str()].join("")]).creation_flags(CREATE_NO_WINDOW).output();
        match out {
            Err(_) => false,
            Ok(r) => r.status.success(),
        }
    }

    fn unlock_file(&self, p: &String) -> bool {
        let out = Command::new("cmd").args(["/C".into(), ["git lfs unlock ", p.as_str()].join("")]).creation_flags(CREATE_NO_WINDOW).output();
        match out {
            Err(_) => false,
            Ok(r) => r.status.success(),
        }
    }

    fn update(&self) {
        let locks = self.get_raw_locks();
        let mut orphan_tags = vec![];
        for lock in &locks {
            match tag::get_tag(&lock) {
                None => (),
                Some(tag) => {
                    if locks.iter().find(|lock| lock.id == tag.get_target_id()).is_none() {
                        orphan_tags.push(tag);
                    }
                },
            }
        }
        if !orphan_tags.is_empty() {
            for tag in orphan_tags {
                tag.cleanup(self);
            }
            self.update();
        }
    }

    fn unlock_id(&self, id: u32) -> bool {
        let out = Command::new("cmd").args(["/C".into(), ["git lfs unlock --id ".into(), id.to_string()].join("")]).creation_flags(CREATE_NO_WINDOW).output();
        match out {
            Err(_) => false,
            Ok(r) => r.status.success(),
        }
    }
}