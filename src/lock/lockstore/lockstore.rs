use crate::lock::LfsLock;
use crate::lock::tag::*;

use std::collections::HashMap;
use std::process::Command;
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

fn normalize_path(p: &String) -> String {
    let s = p.replace("\\", "/");
    match s.strip_prefix("./") {
        None => s,
        Some(stripped) => stripped.to_string(),
    }
}

// A trait for extracting LfsLocks from a repo
pub trait LockStore {

    // Straight pipe from git to untagged locks
    fn get_raw_locks(&self) -> Vec<LfsLock>;

    // Pull down fully tagged and qualified locks
    fn get_locks(&self) -> Vec<LfsLock> {
        let locks = self.get_raw_locks();
        let mut real_locks = vec![];
        let mut tags = vec![];
        for lock in locks {
            match tag::get_tag(&lock) {
                None => real_locks.push(lock),
                Some(tag) => tags.push(tag),
            }
        }
        for tag in tags {
            match real_locks.iter_mut().find(|lock| lock.id == tag.get_target_id()) {
                None => (),
                Some(l) => tag.apply(l),
            }
        }
        real_locks
    }

    // Pull down fully tagged and qualified lock
    fn get_lock_file(&self, p: &String) -> Option<LfsLock> {
        self.get_locks().into_iter().find(|lock| normalize_path(&lock.file) == normalize_path(p))
    }

    // Pull down fully tagged and qualified lock
    fn get_lock_id(&self, id: u32) -> Option<LfsLock> {
        self.get_locks().into_iter().filter(|lock| lock.id == id).last()
    }

    /* Find pending actions and execute them. e.g. cleaning up orphaned tags or deleting locks when
    the owning branch no longer exists */
    fn update(&self);

    // Lock a file
    fn lock_file(&self, p: &String) -> bool  {
        let lock = ["git lfs lock", p].join(" ");
        let cmd = Command::new("cmd").args(["/C", &lock]).creation_flags(CREATE_NO_WINDOW).output();
        match cmd {
            Err(e) => {
                println!("Error: {}", e.to_string());
                false
            },
            Ok(r) => r.status.success(),
        }
    }

    fn lock_file_fast(&self, p: &String) {
        self.lock_file(p);
    }

    // locks a file, then returns the newly created lock
    fn lock_file_fetch(&self, p: &String) -> Option<LfsLock> {
        match self.lock_file(p) {
            true => self.get_lock_file(p),
            false => None
        }
    }

    // lock a real file, not an arbitrary path
    fn lock_real_file(&self, p: &String) -> Option<LfsLock> {
        match self.lock_file_fetch(p) {
            None => None,
            Some(lock) => {
                let bt = branchtag::for_lock(&lock);
                let dt = dirtag::for_lock(&lock);
                self.lock_file_fast(&bt.get_lock_string());
                self.lock_file_fast(&dt.get_lock_string());
                Some(lock)
            }
        }
    }

    fn unlock_file(&self, p: &String) -> bool;

    fn unlock_file_fast(&self, p: &String) {
        self.unlock_file(p);
    }

    fn unlock_id(&self, id: u32) -> bool;

    fn unlock_id_fast(&self, id: u32) {
        self.unlock_id(id);
    }

}