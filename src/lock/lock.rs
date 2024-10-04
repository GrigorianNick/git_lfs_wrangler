use core::fmt;
use std::process::Command;
use std::collections::HashMap;
use crate::lock::tag::tag::Tag;

use super::tag;


fn normalize_path(p: &String) -> String {
    let s = p.replace("\\", "/");
    match s.strip_prefix("./") {
        None => s,
        Some(stripped) => stripped.to_string(),
    }
}

pub struct LfsLock {
    pub file: String,
    pub owner: String,
    pub id: u32,
    pub branch: Option<String>,
    pub dir: Option<String>,
    pub queue: Vec<String>,
    pub tags: Vec<Box<dyn Tag>>,
}

impl LfsLock {
    pub fn from_line(line: String) -> Option<LfsLock> {
        let fields: Vec<&str> = line.split_whitespace().filter(|&s| !s.is_empty()).collect();
        match fields.len() {
            3 => Some(LfsLock::new(fields[0].to_string(), fields[1].to_string(), fields[2].to_string(), None)),
            _ => None,
        }
    }

    pub fn new(file: String, owner: String, id: String, branch: Option<String>) -> Self {
        let _ = id.strip_prefix("ID:");
        let id_num = match id.trim_start_matches("ID:").parse::<u32>() {
            Ok(val) => val,
            Err(_) => 0,
        };
        LfsLock{
            file: file,
            owner: owner,
            id: id_num,
            branch: branch,
            dir: None,
            queue: vec![],
            tags: vec![],
        }
    }

    pub fn unlock(&self) {
        let unlock = ["git lfs unlock -i", &self.id.to_string()].join(" ");
        let _ = Command::new("cmd").args(["/C", &unlock]).output();
        match &self.branch {
            None => (),
            Some(branch) => {
                let tag_name = [self.id.to_string(), branch.to_string()].join("___");
                let unlock_tag = ["git lfs unlock", &tag_name].join(" ");
                let _ = Command::new("cmd").args(["/C", &unlock_tag]).output();
            }
        }
    }

    pub fn unlock_file(p: &String) -> bool {
        let lock = ["git lfs unlock", p].join(" ");
        println!("Running command: {}", lock);
        let cmd = Command::new("cmd").args(["/C", &lock]).output();
        match cmd {
            Err(_) => false,
            Ok(e) => e.status.success(),
        }
    }
}

impl fmt::Display for LfsLock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.branch {
            Some(branch_name) => write!(f, "file: {}; owner: {}; id: {}; branch: {}; queue: {:?}", self.file, self.owner, self.id, branch_name, self.queue),
            None => write!(f, "file: {}; owner: {}; id: {}; branch: {}; queue: {:?}", self.file, self.owner, self.id, "None detected", self.queue),
        }
    }
}

pub struct LockStore {
    locks: Vec<LfsLock>,
    orphan_tags: Vec<Box<dyn Tag>>,
}

impl Default for LockStore {
    fn default() -> Self {
        LockStore {
            locks: vec![],
            orphan_tags: vec![],
        }
    }
}

impl LockStore {

    pub fn new() -> Self {
        let mut store = LockStore::default();
        store.update_locks();
        store
    }

    // Fetches raw locks
    pub fn fetch_raw_locks(&self) -> Vec<LfsLock> {
        let out = Command::new("cmd").args(["/C", "git lfs locks"]).output().expect("Failed to execute process");
        let out = String::from_utf8_lossy(&out.stdout).to_string();
        let lines: Vec<&str> = out.split("\n").filter(|&s| !s.is_empty()).collect();
        let locks: Vec<LfsLock> = lines.iter().map(|&l| LfsLock::from_line(l.to_string()).unwrap()).collect();
        locks
    }

    pub fn update_locks(&mut self) {
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
            tag.cleanup(self);
        }
        if recurse {
            self.orphan_tags.clear();
            self.update_locks();
        }
    }

    // locks a file, then returns the newly created lock
    pub fn lock_file_fetch(&self, p: &String) -> Option<LfsLock>{
        if self.lock_file(p) {
            let locks = self.fetch_raw_locks().into_iter().filter(|l| normalize_path(&l.file) == normalize_path(p));
            locks.last()
        } else {
            None
        }
    }

    pub fn lock_file(&self, p: &String) -> bool {
        let lock = ["git lfs lock", p].join(" ");
        let cmd = Command::new("cmd").args(["/C", &lock]).output();
        match cmd {
            Err(_) => false,
            Ok(r) => {
                println!("Lock exit code:{}", r.status.success());
                r.status.success()
            }
        }
    }

    // lock a real file, not an arbitrary path
    pub fn lock_real_file(&self, p: &String) -> bool {
        match self.lock_file_fetch(p) {
            None => false,
            Some(lock) => {
                let bt = tag::branchtag::for_lock(&lock);
                let dt = tag::dirtag::for_lock(&lock);
                bt.save(self);
                dt.save(self);
                true
            }
        }
    }

    pub fn unlock_id(&self, id: u32) {
        for lock in &self.locks {
            if lock.id == id {
                println!("Unlocking file:id; {}:{}", &lock.file, id);
                LfsLock::unlock_file(&lock.file);
            }
        }
    }

    pub fn tag(&mut self, tag: Box<dyn tag::Tag>) {
        for lock in &mut self.locks {
            if lock.id == tag.get_target_id() {
                tag.apply(lock);
            }
        }
        tag.save(self);
    }

    pub fn get_lock_id(&self, id: u32) -> Option<&LfsLock> {
        self.locks.iter().filter(|lock| lock.id == id).last()
    }

    pub fn get_locks(&self) -> Vec<&LfsLock> {
        self.locks.iter().collect()
    }
}