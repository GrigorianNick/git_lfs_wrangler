use crate::git;
use crate::lock::LfsLock;
use crate::lock::tag::*;

use std::process::Command;
use std::os::windows::process::CommandExt;

use super::LockStore;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct MonothreadLockStore {
}

impl Default for MonothreadLockStore {
    fn default() -> Self {
        MonothreadLockStore {
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

    fn lock_file_fetch(&self, p: &String) -> Option<LfsLock> {
        let lock = ["git lfs lock", p, "--json"].join(" ");
        let cmd = Command::new("cmd").args(["/C", &lock]).creation_flags(CREATE_NO_WINDOW).output();
        match cmd {
            Err(e) => {
                println!("Error: {}", e.to_string());
                None
            },
            Ok(r) => {
                if !r.status.success() {
                    return None;
                }
                let json_str = std::str::from_utf8(&r.stdout).expect("Failed to get stdout");
                let json: serde_json::Value = serde_json::from_str(json_str).expect("Failed to parse json");
                let id = str::parse::<u32>(json[0]["id"].as_str().unwrap()).expect("Failed to parse value");
                Some(LfsLock{
                    file: p.clone(),
                    owner: git::get_lfs_user(),
                    id: id as u32,
                    branch: None,
                    dir: None,
                    queue: vec![],
                })
            }
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
        let user = git::get_lfs_user();
        let locks = self.get_raw_locks();
        let mut orphan_tags = vec![];
        for lock in &locks {
            if lock.owner != user {
                continue;
            }
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