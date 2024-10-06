use crate::{git, lock::{lock, LfsLock}};
use super::Tag;
use crate::lock::LockStore;

use regex::Regex;

pub struct QueueTag {
    target_id: u32,
    target_file: String,
    queue_owner: String,
}

pub fn for_lock(lock: &LfsLock, store: &LockStore) -> Box<QueueTag> {
    Box::new(
        QueueTag {
            target_id: lock.id,
            target_file: lock.file.clone(),
            queue_owner: git::get_lfs_user(store),
        }
    )
}

impl QueueTag {
    pub fn from_lock(lock: &LfsLock) -> Option<Box<dyn Tag>> {
        let re = Regex::new(r"Q(?<id>[0-9]+)_(?<owner>.+)___(?<file>.*)").expect("Regex failed to compile");
        match re.captures(&lock.file) {
            None => None,
            Some(c) =>  {
                match (c.name("id"), c.name("owner"), c.name("file")) {
                    (Some(id), Some(owner), Some(f)) => Some(Box::new(QueueTag{
                        target_id: id.as_str().parse().expect("failed to parse int"),
                        target_file: f.as_str().to_string(),
                        queue_owner: owner.as_str().to_string(),
                    })),
                    _ => None,
                }
            }
        }
    }
}

impl Tag for QueueTag {

    fn get_lock_string(&self) -> String {
        ["Q", self.target_id.to_string().as_str(), "_", self.queue_owner.as_str(), "___", self.target_file.as_str()].join("")
    }

    fn apply(&self, lock: &mut LfsLock) {
        lock.queue.push(self.queue_owner.clone());
    }

    fn get_target_id(&self) -> u32 {
        self.target_id
    }

    fn cleanup(&self, store: &LockStore) {
        if self.queue_owner != git::get_lfs_user(store) {
            return
        }
        let target_lock = store.get_lock_id(self.get_target_id());
        match target_lock {
            // target lock doesn't exist, grab it
            None => {
                println!("Queue target doesn't exist");
                if store.lock_real_file(&self.target_file) {
                    println!("Grabbed lock. Released tag lock? {}", lock::LfsLock::unlock_file(&self.get_lock_string()));
                    //lock::LfsLock::unlock_file(&self.get_lock_string());
                } else {
                    println!("Failed to grab file, relocking");
                    println!("Released tag lock? {}", lock::LfsLock::unlock_file(&self.get_lock_string()));
                    //lock::LfsLock::unlock_file(&self.get_lock_string());
                    match store.get_lock_file(&self.target_file) {
                        // Nonsense case?
                        None => (),
                        Some(lock) => {
                            println!("Found queue target for new lock");
                            let new_tag = for_lock(lock, store);
                            new_tag.save(store);
                        }
                    }
                }
            }
            // target exists, if we own it nuke ourselves
            Some(l) => {
                println!("Queue target exists. Owners: {}:{}", l.owner, git::get_lfs_user(store));
                if l.owner == git::get_lfs_user(store) {
                    lock::LfsLock::unlock_file(&self.get_lock_string());
                }
            }
        }
    }
}