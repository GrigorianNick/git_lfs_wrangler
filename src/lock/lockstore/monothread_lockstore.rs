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
    orphan_tags: Vec<Box<dyn Tag>>,
}

impl Default for MonothreadLockStore {
    fn default() -> Self {
        MonothreadLockStore {
            locks: vec![],
            orphan_tags: vec![],
        }
    }
}

impl MonothreadLockStore {

    pub fn new() -> Box<Self> {
        let mut store = Box::new(MonothreadLockStore::default());
        store.update_locks();
        store
    }
}

impl LockStore for MonothreadLockStore {

    // Fetches raw locks
    fn fetch_raw_locks(&self) -> Vec<LfsLock> {
        let out = Command::new("cmd").args(["/C", "git lfs locks"]).creation_flags(CREATE_NO_WINDOW).output().expect("Failed to execute process");
        let out = String::from_utf8_lossy(&out.stdout).to_string();
        let lines: Vec<&str> = out.split("\n").filter(|&s| !s.is_empty()).collect();
        let locks: Vec<LfsLock> = lines.iter().map(|&l| LfsLock::from_line(l.to_string()).unwrap()).collect();
        locks
    }

    fn update_locks(self: &mut Self) {
        let locks = self.fetch_raw_locks();

        let mut lock_map = HashMap::<u32, LfsLock>::new();
        let mut tags = Vec::<Box<dyn tag::Tag>>::new();
        for lock in locks {
            match tag::get_tag(&lock) {
                None => {
                    lock_map.insert(lock.id, lock);
                }
                Some(tag) => {
                    tags.push(tag);
                },
            }
        }
        for tag in tags {
            match lock_map.get_mut(&tag.get_target_id()) {
                None => self.orphan_tags.push(tag),
                Some(lock) => {
                    tag.apply(lock);
                },
            }
        }
        let mut recurse = false;
        self.locks = lock_map.into_values().collect();
        for tag in &self.orphan_tags {
            recurse = true;
            //tag.cleanup(self);
        }
        if recurse {
            self.orphan_tags.clear();
            self.update_locks();
        }
    }

    // locks a file, then returns the newly created lock
    fn lock_file_fetch(&self, p: &String) -> Option<&LfsLock>{
        println!("Attempting to lock fetch:{}", p);
        if self.lock_file(p) {
            self.get_lock_file(p)
            //let locks = self.fetch_raw_locks().into_iter().filter(|l| normalize_path(&l.file) == normalize_path(p));
            //locks.last()
        } else {
            None
        }
    }

    fn lock_file(&self, p: &String) -> bool {
        let lock = ["git lfs lock", p].join(" ");
        println!("Locking file {} with {}", p, lock);
        let cmd = Command::new("cmd").args(["/C", &lock]).creation_flags(CREATE_NO_WINDOW).output();
        match cmd {
            Err(e) => {
                println!("Error: {}", e.to_string());
                false
            },
            Ok(r) => {
                println!("Success for: {}", p);
                r.status.success()
            }
        }
    }

    fn unlock_id(&self, id: u32) {
        for lock in &self.locks {
            if lock.id == id {
                LfsLock::unlock_file(&lock.file);
            }
        }
    }

    fn tag(&mut self, tag: Box<dyn tag::Tag>) {
        for lock in &mut self.locks {
            if lock.id == tag.get_target_id() {
                tag.apply(lock);
            }
        }
        self.lock_file(&tag.get_lock_string());
    }

    fn get_lock_id(&self, id: u32) -> Option<&LfsLock> {
        self.locks.iter().filter(|lock| lock.id == id).last()
    }

    fn get_lock_id_mut(&mut self, id: u32) -> Option<&mut LfsLock> {
        self.locks.iter_mut().filter(|lock| lock.id == id).last()
    }

    fn get_lock_file(&self, file: &String) -> Option<&LfsLock> {
        self.locks.iter().filter(|lock| {
            println!("Comparing lock file paths: {}:{}", lock.file, *file);
            lock.file == *file}
        ).last()
    }

    fn get_locks(&self) -> Vec<&LfsLock> {
        self.locks.iter().collect()
    }
}