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

pub trait LockStore {

    // Fetches raw locks
    fn fetch_raw_locks(&self) -> Vec<LfsLock>;

    fn update_locks(&mut self);

    // locks a file, then returns the newly created lock
    fn lock_file_fetch(&self, p: &String) -> Option<&LfsLock>;

    fn lock_file(&self, p: &String) -> bool;

    // lock a real file, not an arbitrary path
    fn lock_real_file(&self, p: &String) -> bool {
        match self.lock_file_fetch(p) {
            None => false,
            Some(lock) => {
                let bt = branchtag::for_lock(&lock);
                let dt = dirtag::for_lock(&lock);
                self.lock_file(&bt.get_lock_string());
                self.lock_file(&dt.get_lock_string());
                true
            }
        }
    }

    fn unlock_id(&self, id: u32);

    fn tag(&mut self, tag: &dyn Tag) {
        match self.get_lock_id_mut(tag.get_target_id()) {
            None => (),
            Some(mut lock) => tag.apply(lock),
        }
        self.lock_file(&tag.get_lock_string());
        //tag.save(self);
        /*for lock in &mut self.locks {
            if lock.id == tag.get_target_id() {
                tag.apply(lock);
            }
        }
        tag.save(self);*/
    }

    fn get_lock_id(&self, id: u32) -> Option<&LfsLock>;

    fn get_lock_id_mut(&mut self, id: u32) -> Option<&mut LfsLock>;

    fn get_lock_file(&self, file: &String) -> Option<&LfsLock> {
        self.get_locks().into_iter().filter(|lock| lock.file == *file).last()
        //self.locks.iter().filter(|lock| lock.file == *file).last()
    }

    fn get_locks(&self) -> Vec<&LfsLock>;
}