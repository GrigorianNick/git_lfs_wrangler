use crate::lock::{lock, LfsLock, LockStore};
use crate::lock::tag::Tag;

use regex::Regex;

pub struct BranchTag {
    branch: String,
    target_id: u32,
}

impl BranchTag {
    pub fn from_lock(lock: &LfsLock) -> Option<Box<dyn Tag>> {
        let re = Regex::new("B(?<id>[0-9]+)___(?<branch>.*)").unwrap();
        match re.captures(&lock.file) {
            None => None,
            Some(c) => {
                match (c.name("id"), c.name("branch")) {
                    (Some(id), Some(branch)) => {
                        Some(Box::new(
                            BranchTag {
                                branch: branch.as_str().to_string(),
                                target_id: id.as_str().parse::<u32>().unwrap(),
                            }
                        ))
                    },
                    _ => None,
                }
            },
        }
    }
}

pub fn for_lock(lock: &LfsLock) -> Box<dyn Tag> {
    Box::new(
        BranchTag{
            branch: crate::git::get_branch(),
            target_id: lock.id,
        })
}

impl Tag for BranchTag {

    fn save(&self, store: &LockStore) {
        let lock = ["B", self.get_target_id().to_string().as_str(), "___", self.branch.as_str()].join("");
        println!("Tagginb branch:{}", &lock);
        store.lock_file(&lock);
    }

    fn apply(&self, lock: &mut LfsLock) {
        lock.branch = Some(self.branch.clone());
    }

    fn get_target_id(&self) -> u32 {
        self.target_id
    }

    fn cleanup(&self, _store: &LockStore) {
        println!("Cleaning up branchtag!");
        let lock = ["B", self.get_target_id().to_string().as_str(), "___", self.branch.as_str()].join("");
        lock::LfsLock::unlock_file(&lock);
    }
}