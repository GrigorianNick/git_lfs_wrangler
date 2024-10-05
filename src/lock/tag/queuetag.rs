use crate::{git, lock::{lock, LfsLock}};
use super::Tag;
use crate::lock::LockStore;

use regex::Regex;

pub struct QueueTag {
    target_id: u32,
    target_file: String,
    queue_owner: String,
}

pub fn for_lock(lock: &LfsLock) -> Box<QueueTag> {
    Box::new(
        QueueTag {
            target_id: lock.id,
            target_file: lock.file.clone(),
            queue_owner: git::get_user(),
        }
    )
}

impl QueueTag {
    pub fn from_lock(lock: &LfsLock) -> Option<Box<dyn Tag>> {
        let re = Regex::new(r"Q(?<id>[0-9]+)___(?<file>.*)").expect("Regex failed to compile");
        match re.captures(&lock.file) {
            None => None,
            Some(c) =>  {
                match (c.name("id"), c.name("file")) {
                    (Some(id), Some(f)) => Some(Box::new(QueueTag{
                        target_id: id.as_str().parse().expect("failed to parse int"),
                        target_file: f.as_str().to_string(),
                        queue_owner: lock.owner.clone(),
                    })),
                    _ => None,
                }
            }
        }
    }
}

impl Tag for QueueTag {
    fn save(&self, store: &LockStore) {
        let lock_path = ["Q", self.target_id.to_string().as_str(), "___", self.target_file.as_str()].join("");
        store.lock_file(&lock_path);
    }

    fn apply(&self, lock: &mut LfsLock) {
        lock.queue.push(self.queue_owner.clone());
    }

    fn get_target_id(&self) -> u32 {
        self.target_id
    }

    fn cleanup(&self, store: &LockStore) {
        let target_lock = store.get_lock_id(self.get_target_id());
        match target_lock {
            // target lock doesn't exist, grab it
            None => {
                if store.lock_real_file(&self.target_file) {
                    let lock_path = ["Q", self.target_id.to_string().as_str(), "___", self.target_file.as_str()].join("");
                    lock::LfsLock::unlock_file(&lock_path);
                }
            }
            // target exists, if we own it nuke ourselves
            Some(l) => {
                if l.owner == git::get_user() {
                    let lock_path = ["Q", self.target_id.to_string().as_str(), "___", self.target_file.as_str()].join("");
                    lock::LfsLock::unlock_file(&lock_path);
                }
            }
        }
    }
}